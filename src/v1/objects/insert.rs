use crate::{
    common::{Conditionals, PredefinedAcl, Projection, StandardQueryParameters},
    error::Error,
    response::ApiResponse,
    types::{BucketName, ObjectIdentifier, ObjectName},
};
#[cfg(feature = "async-multipart")]
use futures_util::{
    io::{AsyncRead, Result as FuturesResult},
    task::{Context, Poll},
    Stream,
};
#[cfg(feature = "async-multipart")]
use pin_utils::unsafe_pinned;
#[cfg(feature = "async-multipart")]
use std::pin::Pin;
use std::{convert::TryFrom, io};

/// Optional parameters when inserting an object.
/// See [here](https://cloud.google.com/storage/docs/json_api/v1/objects/insert#parameters)
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InsertObjectOptional<'a> {
    #[serde(flatten)]
    pub standard_params: StandardQueryParameters<'a>,
    /// The Content-Type of the object, defaults to `application/octet-stream`.
    #[serde(skip)]
    pub content_type: Option<&'a str>,
    /// If set, sets the contentEncoding property of the final object to
    /// this value. Setting this parameter is equivalent to setting the
    /// `contentEncoding` metadata property. This can be useful when
    /// uploading an object with uploadType=media to indicate the
    /// encoding of the content being uploaded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<&'a str>,
    #[serde(flatten)]
    pub conditionals: Conditionals,
    /// Resource name of the Cloud KMS key that will be used to encrypt
    /// the object. Overrides the object metadata's kms_key_name value, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_name: Option<&'a str>,
    /// Apply a predefined set of access controls to this object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predefined_acl: Option<PredefinedAcl>,
    /// Set of properties to return. Defaults to `noAcl`, unless the object
    /// resource specifies the acl property, when it defaults to full.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<Projection>,
    /// The project to be billed for this request. Required for Requester Pays buckets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_project: Option<&'a str>,
}

/// The response from an insert request is the Object [metadata](https://cloud.google.com/storage/docs/json_api/v1/objects#resource)
/// for the newly inserted Object
pub struct InsertResponse {
    pub metadata: super::Metadata,
}

impl ApiResponse<&[u8]> for InsertResponse {}
impl ApiResponse<bytes::Bytes> for InsertResponse {}

impl<B> TryFrom<http::Response<B>> for InsertResponse
where
    B: AsRef<[u8]>,
{
    type Error = Error;

    fn try_from(response: http::Response<B>) -> Result<Self, Self::Error> {
        let (_parts, body) = response.into_parts();
        let metadata: super::Metadata = serde_json::from_slice(body.as_ref())?;
        Ok(Self { metadata })
    }
}

const MULTI_PART_SEPARATOR: &[u8] = b"--tame_gcs\n";
const MULTI_PART_SUFFIX: &[u8] = b"\n--tame_gcs--";
const MULTI_PART_CT: &[u8] = b"content-type: application/json; charset=utf-8\n\n";

enum MultipartPart {
    Prefix,
    Body,
    Suffix,
    End,
}

impl MultipartPart {
    fn next(&mut self) {
        match self {
            MultipartPart::Prefix => *self = MultipartPart::Body,
            MultipartPart::Body => *self = MultipartPart::Suffix,
            MultipartPart::Suffix => *self = MultipartPart::End,
            MultipartPart::End => unreachable!(),
        }
    }
}

struct MultipartCursor {
    position: usize,
    part: MultipartPart,
}

/// A multipart payload that should be used as the body of a multipart
/// insert request
pub struct Multipart<B> {
    body: B,
    prefix: bytes::Bytes,
    body_len: u64,
    total_len: u64,
    cursor: MultipartCursor,
}

impl<B> Multipart<B> {
    #[cfg(feature = "async-multipart")]
    unsafe_pinned!(body: B);

    /// Wraps some body content and its metadata into a Multipart suitable for being
    /// sent as an HTTP request body, the body will need to implement `std::io::Read`
    /// to be able to be used as intended.
    pub fn wrap(body: B, body_length: u64, metadata: &super::Metadata) -> Result<Self, Error> {
        use bytes::BufMut;

        const CT_HN: &[u8] = b"content-type: ";

        // I wonder if this counts as sansio...
        let serialized_metadata = serde_json::to_vec(metadata)?;
        let content_type = metadata
            .content_type
            .as_deref()
            .unwrap_or("application/octet-stream")
            .as_bytes();

        let metadata = &serialized_metadata[..];

        // Example request from https://cloud.google.com/storage/docs/json_api/v1/how-tos/multipart-upload
        // POST https://www.googleapis.com/upload/storage/v1/b/myBucket/o?uploadType=multipart HTTP/1.1
        // Authorization: Bearer [YOUR_AUTH_TOKEN]
        // Content-Type: multipart/related; boundary=foo_bar_baz
        // Content-Length: [NUMBER_OF_BYTES_IN_ENTIRE_REQUEST_BODY]

        // --foo_bar_baz
        // Content-Type: application/json; charset=UTF-8

        // {
        // "name": "myObject"
        // }

        // --foo_bar_baz
        // Content-Type: image/jpeg

        // [JPEG_DATA]
        // --foo_bar_baz--
        let prefix_len = MULTI_PART_SEPARATOR.len()
            + MULTI_PART_CT.len()
            + metadata.len()
            + 1
            + MULTI_PART_SEPARATOR.len()
            + CT_HN.len()
            + content_type.len()
            + 2;

        let prefix = {
            let mut prefix = bytes::BytesMut::with_capacity(prefix_len);
            prefix.put_slice(MULTI_PART_SEPARATOR);
            prefix.put_slice(MULTI_PART_CT);
            prefix.put_slice(metadata);
            prefix.put_slice(b"\n");
            prefix.put_slice(MULTI_PART_SEPARATOR);
            prefix.put_slice(CT_HN);
            prefix.put_slice(content_type);
            prefix.put_slice(b"\n\n");

            prefix.freeze()
        };

        let total_len = prefix_len as u64 + body_length + MULTI_PART_SUFFIX.len() as u64;

        Ok(Self {
            body,
            prefix,
            body_len: body_length,
            total_len,
            cursor: MultipartCursor {
                position: 0,
                part: MultipartPart::Prefix,
            },
        })
    }

    /// The total length (Content-Length) of this multipart body
    pub fn total_len(&self) -> u64 {
        self.total_len
    }
}

impl<B> io::Read for Multipart<B>
where
    B: io::Read,
{
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        use std::cmp::min;
        let mut total_copied = 0;

        while total_copied < buffer.len() {
            let buf = &mut buffer[total_copied..];

            let (copied, len) = match self.cursor.part {
                MultipartPart::Prefix => {
                    let to_copy = min(buf.len(), self.prefix.len() - self.cursor.position);

                    buf[..to_copy].copy_from_slice(
                        &self.prefix[self.cursor.position..self.cursor.position + to_copy],
                    );

                    (to_copy, self.prefix.len())
                }
                MultipartPart::Body => {
                    let copied = self.body.read(buf)?;
                    (copied, self.body_len as usize)
                }
                MultipartPart::Suffix => {
                    let to_copy = min(buf.len(), MULTI_PART_SUFFIX.len() - self.cursor.position);

                    buf[..to_copy].copy_from_slice(
                        &MULTI_PART_SUFFIX[self.cursor.position..self.cursor.position + to_copy],
                    );

                    (to_copy, MULTI_PART_SUFFIX.len())
                }
                MultipartPart::End => return Ok(total_copied),
            };

            self.cursor.position += copied;
            total_copied += copied;

            if self.cursor.position == len {
                self.cursor.part.next();
                self.cursor.position = 0;
            }
        }

        Ok(total_copied)
    }
}

#[cfg(feature = "async-multipart")]
impl<B: AsyncRead + Unpin> AsyncRead for Multipart<B> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<FuturesResult<usize>> {
        use std::cmp::min;
        let mut total_copied = 0;

        let (copied, len) = match self.cursor.part {
            MultipartPart::Prefix => {
                let to_copy = min(buf.len(), self.prefix.len() - self.cursor.position);

                buf[..to_copy].copy_from_slice(
                    &self.prefix[self.cursor.position..self.cursor.position + to_copy],
                );

                (to_copy, self.prefix.len())
            }
            MultipartPart::Body => {
                let copied = match self.as_mut().body().poll_read(cx, buf) {
                    Poll::Ready(Ok(copied)) => copied,
                    other => return other,
                };
                (copied, self.body_len as usize)
            }
            MultipartPart::Suffix => {
                let to_copy = min(buf.len(), MULTI_PART_SUFFIX.len() - self.cursor.position);

                buf[..to_copy].copy_from_slice(
                    &MULTI_PART_SUFFIX[self.cursor.position..self.cursor.position + to_copy],
                );

                (to_copy, MULTI_PART_SUFFIX.len())
            }
            MultipartPart::End => return Poll::Ready(Ok(0)),
        };

        self.cursor.position += copied;
        total_copied += copied;

        if self.cursor.position == len {
            self.cursor.part.next();
            self.cursor.position = 0;
        }

        Poll::Ready(Ok(total_copied))
    }
}

#[cfg(feature = "async-multipart")]
impl Stream for Multipart<bytes::Bytes> {
    type Item = bytes::Bytes;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(match self.cursor.part {
            MultipartPart::Prefix => {
                self.cursor.part.next();
                Some(self.prefix.clone())
            }
            MultipartPart::Body => {
                self.cursor.part.next();
                Some(self.body.clone())
            }
            MultipartPart::Suffix => {
                self.cursor.part.next();
                Some(bytes::Bytes::from(MULTI_PART_SUFFIX))
            }
            MultipartPart::End => None,
        })
    }
}

impl super::Object {
    /// Stores a new object and metadata.
    ///
    /// * Maximum file size: `5TB`
    /// * Accepted Media MIME types: `*/*`
    ///
    /// Required IAM Permissions: `storage.objects.create`, `storage.objects.delete`
    ///
    /// Note: `storage.objects.delete` is only needed if an object with the same
    /// name already exists.
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/json_api/v1/objects/insert)
    pub fn insert_simple<'a, OID, B>(
        id: &OID,
        content: B,
        length: u64,
        optional: Option<InsertObjectOptional<'_>>,
    ) -> Result<http::Request<B>, Error>
    where
        OID: ObjectIdentifier<'a> + ?Sized,
    {
        let mut uri = format!(
            "https://www.googleapis.com/upload/storage/v1/b/{}/o?name={}&uploadType=media",
            percent_encoding::percent_encode(id.bucket().as_ref(), crate::util::PATH_ENCODE_SET,),
            percent_encoding::percent_encode(id.object().as_ref(), crate::util::QUERY_ENCODE_SET,),
        );

        let query = optional.unwrap_or_default();

        let req_builder = http::Request::builder()
            .header(
                http::header::CONTENT_TYPE,
                http::header::HeaderValue::from_str(
                    query.content_type.unwrap_or("application/octet-stream"),
                )
                .map_err(http::Error::from)?,
            )
            .header(http::header::CONTENT_LENGTH, length);

        let query_params = serde_urlencoded::to_string(query)?;
        if !query_params.is_empty() {
            uri.push('&');
            uri.push_str(&query_params);
        }

        Ok(req_builder.method("POST").uri(uri).body(content)?)
    }

    /// Stores a new object and metadata.
    ///
    /// * Maximum file size: `5TB`
    /// * Accepted Media MIME types: `*/*`
    ///
    /// This method differs from `insert_simple` in that it performs a
    /// [multipart upload](https://cloud.google.com/storage/docs/json_api/v1/how-tos/multipart-upload)
    /// which allows you specify both the object data and its metadata in a single
    /// request, instead of having to do an additional request to set the metadata.
    ///
    /// **NOTE**: You **must** specify the `name` field in the metadata provided to this function
    /// with a valid object name. Only the `content_type` specified in `metadata` will be used,
    /// the `content_type` in `optional` will be ignored.
    ///
    /// Required IAM Permissions: `storage.objects.create`, `storage.objects.delete`
    ///
    /// Note: `storage.objects.delete` is only needed if an object with the same
    /// name already exists.
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/json_api/v1/objects/insert)
    pub fn insert_multipart<B>(
        bucket: &BucketName<'_>,
        content: B,
        length: u64,
        metadata: &super::Metadata,
        optional: Option<InsertObjectOptional<'_>>,
    ) -> Result<http::Request<Multipart<B>>, Error> {
        // Since the user can specify the name in the metadata, we just always
        // use that
        match metadata.name {
            Some(ref name) => ObjectName::try_from(name.as_ref())?,
            None => {
                return Err(Error::InvalidLength {
                    len: 0,
                    min: 1,
                    max: 1024,
                })
            }
        };

        let mut uri = format!(
            "https://www.googleapis.com/upload/storage/v1/b/{}/o?uploadType=multipart",
            percent_encoding::percent_encode(bucket.as_ref(), crate::util::PATH_ENCODE_SET,),
        );

        let query = optional.unwrap_or_default();

        let multipart = Multipart::wrap(content, length, metadata)?;

        let req_builder = http::Request::builder()
            .header(
                http::header::CONTENT_TYPE,
                http::header::HeaderValue::from_static("multipart/related; boundary=tame_gcs"),
            )
            .header(http::header::CONTENT_LENGTH, multipart.total_len());

        let query_params = serde_urlencoded::to_string(query)?;
        if !query_params.is_empty() {
            uri.push('&');
            uri.push_str(&query_params);
        }

        Ok(req_builder.method("POST").uri(uri).body(multipart)?)
    }
}
