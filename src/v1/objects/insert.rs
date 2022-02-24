use crate::{
    common::{Conditionals, PredefinedAcl, Projection, StandardQueryParameters},
    error::{self, Error},
    response::ApiResponse,
    types::ObjectIdentifier,
};

mod multipart;
mod resumable;

pub use multipart::*;
pub use resumable::*;

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
}
