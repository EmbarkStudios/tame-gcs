use crate::{
    common::{Conditionals, StandardQueryParameters},
    error::Error,
    response::ApiResponse,
    types::ObjectIdentifier,
};

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteObjectOptional<'a> {
    #[serde(flatten)]
    pub standard_params: StandardQueryParameters<'a>,
    /// If present, permanently deletes a specific revision of this object
    /// (as opposed to the latest version, the default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<i64>,
    #[serde(flatten)]
    pub conditionals: Conditionals,
    /// The project to be billed for this request. Required for Requester Pays buckets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_project: Option<&'a str>,
}

pub struct DeleteObjectResponse;

impl ApiResponse<&[u8]> for DeleteObjectResponse {}
impl ApiResponse<bytes::Bytes> for DeleteObjectResponse {}

impl<B> TryFrom<http::Response<B>> for DeleteObjectResponse
where
    B: AsRef<[u8]>,
{
    type Error = Error;

    fn try_from(response: http::Response<B>) -> Result<Self, Self::Error> {
        if response.status() == http::StatusCode::NO_CONTENT {
            Ok(Self)
        } else {
            Err(Self::Error::from(response.status()))
        }
    }
}

impl super::Object {
    /// Deletes an object and its metadata. Deletions are permanent if versioning
    /// is not enabled for the bucket, or if the generation parameter is used.
    ///
    /// Required IAM Permissions: `storage.objects.delete`
    ///
    /// [Complete API documentation](https://cloud.google.com/storage/docs/json_api/v1/objects/delete)
    pub fn delete<'a, OID>(
        id: &OID,
        optional: Option<DeleteObjectOptional<'_>>,
    ) -> Result<http::Request<std::io::Empty>, Error>
    where
        OID: ObjectIdentifier<'a> + ?Sized,
    {
        let mut uri = crate::__make_obj_url!("https://www.googleapis.com/storage/v1/b/{}/o/{}", id);

        let query = optional.unwrap_or_default();
        let query_params = serde_urlencoded::to_string(query)?;
        if !query_params.is_empty() {
            uri.push('?');
            uri.push_str(&query_params);
        }

        let req_builder = http::Request::builder();

        Ok(req_builder
            .method("DELETE")
            .uri(uri)
            .body(std::io::empty())?)
    }
}
