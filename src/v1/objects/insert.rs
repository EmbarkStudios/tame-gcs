use crate::{
    common::{Conditionals, PredefinedAcl, Projection, StandardQueryParameters},
    error::{self, Error},
    response::ApiResponse,
    types::ObjectIdentifier,
};

mod multipart;

pub use multipart::*;

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
