use super::InsertObjectOptional;
use crate::{
    objects::{Metadata, Object},
    types::{BucketName, ObjectName},
    Error,
};
use std::io;

#[cfg(feature = "async-multipart")]
mod async_mp;
#[cfg(feature = "async-multipart")]
pub use async_mp::*;

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
    pin_utils::unsafe_pinned!(body: B);

    /// Wraps some body content and its metadata into a Multipart suitable for being
    /// sent as an HTTP request body, the body will need to implement `std::io::Read`
    /// to be able to be used as intended.
    pub fn wrap(body: B, body_length: u64, metadata: &Metadata) -> Result<Self, Error> {
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

impl Object {
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
        metadata: &Metadata,
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
