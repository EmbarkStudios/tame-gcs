use super::*;
use crate::objects::{Metadata, Object};

#[derive(Clone)]
pub struct ResumableSession(pub http::Uri);

impl From<ResumableSession> for http::Uri {
    fn from(rs: ResumableSession) -> Self {
        rs.0
    }
}

/// The response from an [`Object::init_resumable_insert`] request is the
/// `session_uri`.
pub struct InitResumableInsertResponse {
    pub resumable_session: ResumableSession,
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
                    resumable_session: ResumableSession(session_uri.parse()?),
                }),
                Err(_err) => Err(Error::OpaqueHeaderValue(session_uri.clone())),
            },
            None => Err(Error::UnknownHeader(http::header::LOCATION)),
        }
    }
}

pub enum ResumableInsertResponseMetadata {
    PartialSize(u64),
    Complete(Box<Metadata>),
}

/// The response from an [`resumable_upload`](#method.resumable_upload) request
/// is the enum [`ResumableInsertResponseMetadata`], which would be the size of
/// the object uploaded so far, unless it's the request with last chunk that
/// completes the upload wherein it would be the object
/// [metadata](https://cloud.google.com/storage/docs/json_api/v1/objects#resource).
pub struct ResumableInsertResponse {
    pub metadata: ResumableInsertResponseMetadata,
}

impl ResumableInsertResponse {
    fn try_from_response<B: AsRef<[u8]>>(
        response: http::response::Response<B>,
    ) -> Result<Self, Error> {
        let status = response.status();
        if status.eq(&http::StatusCode::PERMANENT_REDIRECT)
        // Cloud Storage uses 308 (PERMANENT_REDIRECT) in a non-standard way though.
        // See https://cloud.google.com/storage/docs/json_api/v1/status-codes#308_Resume_Incomplete
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
                    if let Ok(message) = std::str::from_utf8(response.body().as_ref()) {
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
        if response.status().as_u16() == 499 {
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

impl Object {
    /// Initiates a resumable upload session.
    ///
    /// * Accepted Media MIME types: `*/*`
    ///
    /// Note: A resumable upload must be completed within a week of being initiated.
    ///
    /// **CAUTION**: Be careful when sharing the resumable session URI, because
    /// it can be used by anyone to upload data to the target bucket without any
    /// further authentication.
    ///  
    /// Required IAM Permissions: `storage.objects.create`, `storage.objects.delete`
    ///
    /// Note: `storage.objects.delete` is only needed if an object with the same
    /// name already exists.
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/performing-resumable-uploads#initiate-session)
    pub fn resumable_insert_init<'a, OID>(
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

    /// Cancels an incomplete resumable upload and prevent any further action for
    /// `session_uri`, which should have been obtained using [`Object::init_resumable_insert`]
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/performing-resumable-uploads#cancel-upload)
    pub fn resumable_cancel(session: ResumableSession) -> Result<http::Request<()>, Error> {
        let req_builder = http::Request::builder().header(http::header::CONTENT_LENGTH, 0u64);

        Ok(req_builder.method("DELETE").uri(session).body(())?)
    }

    /// Performs resumable upload to the specified `session_uri`, which should
    /// have been obtained using [`Object::init_resumable_insert`]
    ///
    /// * Maximum total object size: `5TB`
    ///
    /// There are two ways to upload the object's data:
    /// * For single chunk upload, set `length` to the total size of the object.
    /// * For multiple chunks upload, set `length` to the size of current chunk
    /// that is being uploaded and `Content-Range` header as
    /// `bytes CHUNK_FIRST_BYTE-CHUNK_LAST_BYTE/TOTAL_OBJECT_SIZE` where:
    ///    * `CHUNK_FIRST_BYTE` is the starting byte in the overall object that
    /// the chunk you're uploading contains.
    ///    * `CHUNK_LAST_BYTE` is the ending byte in the overall object that the
    /// chunk you're uploading contains.
    ///    * `TOTAL_OBJECT_SIZE` is the total size of the object you are uploading.
    ///
    /// **NOTE**: `length` should be a multiple of 256KiB, unless it's the last
    /// chunk. If not, the server will not accept all bytes sent in the request.
    /// Also, it is recommended to use at least 8MiB.
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/performing-resumable-uploads#chunked-upload)
    pub fn resumable_append<B>(
        session: ResumableSession,
        content: B,
        length: u64,
    ) -> Result<http::Request<B>, Error> {
        let req_builder = http::Request::builder().header(http::header::CONTENT_LENGTH, length);

        Ok(req_builder.method("PUT").uri(session).body(content)?)
    }
}
