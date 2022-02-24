use crate::{
    common::{Conditionals, PredefinedAcl, Projection, StandardQueryParameters},
    error::{self, Error},
    response::ApiResponse,
    types::{BucketName, ObjectIdentifier, ObjectName},
};
#[cfg(feature = "async-multipart")]
use futures_util::{
    io::{AsyncRead, Result as FuturesResult},
    task::{Context, Poll},
    Stream,
};
use http::StatusCode;
#[cfg(feature = "async-multipart")]
use pin_utils::unsafe_pinned;
#[cfg(feature = "async-multipart")]
use std::pin::Pin;
use std::{convert::TryFrom, io, str};

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

/// The response from an [`insert`](#method.insert) request is the object [metadata](https://cloud.google.com/storage/docs/json_api/v1/objects#resource)
/// for the newly inserted object.
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

/// The response from an [`init_resumable_insert`](#method.init_resumable_insert) request is the `session_uri`.
pub struct InitResumableInsertResponse {
    pub session_uri: String,
}

impl ApiResponse<&[u8]> for InitResumableInsertResponse {}
impl ApiResponse<bytes::Bytes> for InitResumableInsertResponse {}

impl<B> TryFrom<http::Response<B>> for InitResumableInsertResponse
where
    B: AsRef<[u8]>,
{
    type Error = Error;

    fn try_from(response: http::Response<B>) -> Result<Self, Self::Error> {
        let (parts, _body) = response.into_parts();
        match parts.headers.get(http::header::LOCATION) {
            Some(session_uri) => match session_uri.to_str() {
                Ok(session_uri) => Ok(Self {
                    session_uri: session_uri.to_owned(),
                }),
                Err(_err) => Err(Error::OpaqueHeaderValue(session_uri.clone())),
            },
            None => Err(Error::UnknownHeader(http::header::LOCATION)),
        }
    }
}

pub enum ResumableInsertResponseMetadata {
    PartialSize(u64),
    Complete(Box<super::Metadata>),
}

/// The response from an [`resumable_upload`](#method.resumable_upload) request is the enum [`ResumableInsertResponseMetadata`],
/// which would be the size of the object uploaded so far,
/// unless it's the request with last chunk that completes the upload wherein it would be the object [metadata](https://cloud.google.com/storage/docs/json_api/v1/objects#resource).
pub struct ResumableInsertResponse {
    pub metadata: ResumableInsertResponseMetadata,
}

impl ResumableInsertResponse {
    fn try_from_response<B: AsRef<[u8]>>(
        response: http::response::Response<B>,
    ) -> Result<Self, Error> {
        let status = response.status();
        if status.eq(&http::StatusCode::PERMANENT_REDIRECT)
        // Cloud Storage uses 308 (PERMANENT_REDIRECT) in a non-standard way though. See https://cloud.google.com/storage/docs/json_api/v1/status-codes#308_Resume_Incomplete
            || status.eq(&http::StatusCode::OK)
            || status.eq(&http::StatusCode::CREATED)
        {
            Self::try_from(response)
        } else {
            // If we get an error, but with a plain text payload, attempt to deserialize
            // an ApiError from it, otherwise fallback to the simple HttpStatus
            if let Some(ct) = response
                .headers()
                .get(http::header::CONTENT_TYPE)
                .and_then(|ct| ct.to_str().ok())
            {
                if ct.starts_with("text/plain") && !response.body().as_ref().is_empty() {
                    if let Ok(message) = str::from_utf8(response.body().as_ref()) {
                        let api_err = error::ApiError {
                            code: status.into(),
                            message: message.to_owned(),
                            errors: vec![],
                        };
                        return Err(Error::Api(api_err));
                    }
                }
            }
            Err(Error::from(response.status()))
        }
    }
}

impl ApiResponse<&[u8]> for ResumableInsertResponse {
    fn try_from_parts(response: http::response::Response<&[u8]>) -> Result<Self, Error> {
        Self::try_from_response(response)
    }
}

impl ApiResponse<bytes::Bytes> for ResumableInsertResponse {
    fn try_from_parts(response: http::response::Response<bytes::Bytes>) -> Result<Self, Error> {
        Self::try_from_response(response)
    }
}

impl<B> TryFrom<http::Response<B>> for ResumableInsertResponse
where
    B: AsRef<[u8]>,
{
    type Error = Error;

    fn try_from(response: http::Response<B>) -> Result<Self, Self::Error> {
        if response.status().eq(&http::StatusCode::PERMANENT_REDIRECT) {
            let (parts, _body) = response.into_parts();
            let end_pos = match parts.headers.get(http::header::RANGE) {
                Some(range_val) => match range_val.to_str() {
                    Ok(range) => match range.split('-').last() {
                        Some(pos) => {
                            let pos = pos.parse::<u64>();
                            match pos {
                                Ok(pos) => Ok(pos),
                                Err(_err) => Err(Error::OpaqueHeaderValue(range_val.clone())),
                            }
                        }
                        None => Err(Error::UnknownHeader(http::header::RANGE)),
                    },
                    Err(_err) => Err(Error::OpaqueHeaderValue(range_val.clone())),
                },
                None => Err(Error::UnknownHeader(http::header::RANGE)),
            }?;
            Ok(Self {
                metadata: ResumableInsertResponseMetadata::PartialSize(end_pos + 1),
            })
        } else {
            let (_parts, body) = response.into_parts();
            let metadata = Box::new(serde_json::from_slice(body.as_ref())?);
            Ok(Self {
                metadata: ResumableInsertResponseMetadata::Complete(metadata),
            })
        }
    }
}

pub struct CancelResumableInsertResponse;

impl CancelResumableInsertResponse {
    fn try_from_response<B: AsRef<[u8]>>(
        response: http::response::Response<B>,
    ) -> Result<Self, Error> {
        if response.status().eq(&StatusCode::from_u16(499).unwrap()) {
            // See https://cloud.google.com/storage/docs/json_api/v1/status-codes#499_Client_Closed_Request
            Self::try_from(response)
        } else {
            Err(Error::from(response.status()))
        }
    }
}

impl ApiResponse<&[u8]> for CancelResumableInsertResponse {
    fn try_from_parts(response: http::response::Response<&[u8]>) -> Result<Self, Error> {
        Self::try_from_response(response)
    }
}

impl ApiResponse<bytes::Bytes> for CancelResumableInsertResponse {
    fn try_from_parts(response: http::response::Response<bytes::Bytes>) -> Result<Self, Error> {
        Self::try_from_response(response)
    }
}

impl<B> TryFrom<http::Response<B>> for CancelResumableInsertResponse
where
    B: AsRef<[u8]>,
{
    type Error = Error;

    fn try_from(_response: http::Response<B>) -> Result<Self, Self::Error> {
        Ok(Self)
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

    /// Initiates a resumable upload session.
    ///
    /// * Accepted Media MIME types: `*/*`
    ///
    /// Note: A resumable upload must be completed within a week of being initiated.
    ///
    /// **CAUTION**: Be careful when sharing the resumable session URI, because it can be used by anyone to upload data to the target bucket without any further authentication.
    ///  
    /// Required IAM Permissions: `storage.objects.create`, `storage.objects.delete`
    ///
    /// Note: `storage.objects.delete` is only needed if an object with the same
    /// name already exists.
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/performing-resumable-uploads#initiate-session)
    pub fn init_resumable_insert<'a, OID>(
        id: &OID,
        content_type: Option<&str>,
    ) -> Result<http::Request<()>, Error>
    where
        OID: ObjectIdentifier<'a> + ?Sized,
    {
        let uri = format!(
            "https://www.googleapis.com/upload/storage/v1/b/{}/o?uploadType=resumable&name={}",
            percent_encoding::percent_encode(id.bucket().as_ref(), crate::util::PATH_ENCODE_SET,),
            percent_encoding::percent_encode(id.object().as_ref(), crate::util::QUERY_ENCODE_SET,),
        );

        let req_builder = http::Request::builder()
            .header(http::header::CONTENT_LENGTH, 0u64)
            .header(
                http::header::HeaderName::from_static("x-upload-content-type"),
                http::header::HeaderValue::from_str(
                    content_type.unwrap_or("application/octet-stream"),
                )
                .map_err(http::Error::from)?,
            );

        Ok(req_builder.method("POST").uri(uri).body(())?)
    }

    /// Cancels an incomplete resumable upload and prevent any further action for `session_uri`, which should have been obtained using [`init_resumable_insert`](#method.init_resumable_insert).
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/performing-resumable-uploads#cancel-upload)
    pub fn cancel_resumable_insert(session_uri: String) -> Result<http::Request<()>, Error> {
        let req_builder = http::Request::builder().header(http::header::CONTENT_LENGTH, 0u64);

        Ok(req_builder.method("DELETE").uri(session_uri).body(())?)
    }

    /// Performs resumable upload to the specified `session_uri`, which should have been obtained using [`init_resumable_insert`](#method.init_resumable_insert).
    ///
    /// * Maximum total object size: `5TB`
    ///
    /// There are two ways to upload the object's data:
    /// * For single chunk upload, set `length` to the total size of the object.
    /// * For multiple chunks upload, set `length` to the size of current chunk that is being uploaded and `Content-Range` header as `bytes CHUNK_FIRST_BYTE-CHUNK_LAST_BYTE/TOTAL_OBJECT_SIZE` where:
    ///    * `CHUNK_FIRST_BYTE` is the starting byte in the overall object that the chunk you're uploading contains.
    ///    * `CHUNK_LAST_BYTE` is the ending byte in the overall object that the chunk you're uploading contains.
    ///    * `TOTAL_OBJECT_SIZE` is the total size of the object you are uploading.
    ///
    ///     **NOTE**: `length` should be a multiple of 256KiB, unless it's the last chunk. If not, the server will not accept all bytes sent in the request.
    ///     Also, it is recommended to use at least 8MiB.
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/performing-resumable-uploads#chunked-upload)
    pub fn resumable_insert<B>(
        session_uri: String,
        content: B,
        length: u64,
    ) -> Result<http::Request<B>, Error> {
        let req_builder = http::Request::builder().header(http::header::CONTENT_LENGTH, length);

        Ok(req_builder.method("PUT").uri(session_uri).body(content)?)
    }
}
