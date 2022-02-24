use crate::{
    common::{Conditionals, StandardQueryParameters},
    error::Error,
    response::ApiResponse,
    types::ObjectIdentifier,
};

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchObjectOptional<'a> {
    #[serde(flatten)]
    pub standard_params: StandardQueryParameters<'a>,
    #[serde(flatten)]
    pub conditionals: Conditionals,
    /// The project to be billed for this request. Required for Requester Pays buckets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_project: Option<&'a str>,
}

pub struct PatchObjectResponse {
    pub metadata: super::Metadata,
}

impl ApiResponse<&[u8]> for PatchObjectResponse {}
impl ApiResponse<bytes::Bytes> for PatchObjectResponse {}

impl<B> TryFrom<http::Response<B>> for PatchObjectResponse
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
    /// Updates a data blob's associated metadata.
    ///
    /// Required IAM Permissions: `storage.objects.get`, `storage.objects.update`
    ///
    /// [Complete API documentation](https://cloud.google.com/storage/docs/json_api/v1/objects/patch)
    pub fn patch<'a, OID>(
        id: &OID,
        metadata: &super::Metadata,
        optional: Option<PatchObjectOptional<'_>>,
    ) -> Result<http::Request<std::io::Cursor<Vec<u8>>>, Error>
    where
        OID: ObjectIdentifier<'a> + ?Sized,
    {
        let mut uri =
            crate::__make_obj_url!("https://storage.googleapis.com/storage/v1/b/{}/o/{}", id);

        let query = optional.unwrap_or_default();
        let query_params = serde_urlencoded::to_string(query)?;
        if !query_params.is_empty() {
            uri.push('?');
            uri.push_str(&query_params);
        }

        let req_builder = http::Request::builder();

        let md = serde_json::to_vec(&metadata)?;
        let len = md.len();
        let md = std::io::Cursor::new(md);

        Ok(req_builder
            .method("PATCH")
            .header("content-type", "application/json")
            .header("content-length", len)
            .uri(uri)
            .body(md)?)
    }
}
