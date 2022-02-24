use crate::{
    common::{Conditionals, Projection, StandardQueryParameters},
    error::Error,
    response::ApiResponse,
    types::ObjectIdentifier,
};

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetObjectOptional<'a> {
    #[serde(flatten)]
    pub standard_params: StandardQueryParameters<'a>,
    /// If present, selects a specific revision of this object
    /// (as opposed to the latest version, the default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<i64>,
    #[serde(flatten)]
    pub conditionals: Conditionals,
    /// Set of properties to return. Defaults to `noAcl`, unless the object
    /// resource specifies the acl property, when it defaults to full.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<Projection>,
    /// The project to be billed for this request. Required for Requester Pays buckets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_project: Option<&'a str>,
}

pub struct GetObjectResponse {
    pub metadata: super::Metadata,
}

impl ApiResponse<&[u8]> for GetObjectResponse {}
impl ApiResponse<bytes::Bytes> for GetObjectResponse {}

impl<B> TryFrom<http::Response<B>> for GetObjectResponse
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
    /// Gets an object's metadata
    ///
    /// Required IAM Permissions: `storage.objects.get`, `storage.objects.getIamPolicy`*
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/json_api/v1/objects/get)
    pub fn get<'a, OID>(
        id: &OID,
        optional: Option<GetObjectOptional<'_>>,
    ) -> Result<http::Request<std::io::Empty>, Error>
    where
        OID: ObjectIdentifier<'a> + ?Sized,
    {
        let mut uri = crate::__make_obj_url!(
            "https://www.googleapis.com/storage/v1/b/{}/o/{}?alt=json",
            id
        );

        let query = optional.unwrap_or_default();
        let query_params = serde_urlencoded::to_string(query)?;
        if !query_params.is_empty() {
            uri.push('&');
            uri.push_str(&query_params);
        }

        let req_builder = http::Request::builder();

        Ok(req_builder.method("GET").uri(uri).body(std::io::empty())?)
    }
}
