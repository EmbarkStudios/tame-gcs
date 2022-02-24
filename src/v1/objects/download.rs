use crate::{
    common::{Conditionals, Projection, StandardQueryParameters},
    error::Error,
    response::ApiResponse,
    types::ObjectIdentifier,
};
use std::io;

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadObjectOptional<'a> {
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

pub struct DownloadObjectResponse {
    buffer: bytes::Bytes,
}

impl DownloadObjectResponse {
    pub fn consume(self) -> bytes::Bytes {
        self.buffer
    }
}

impl ApiResponse<bytes::Bytes> for DownloadObjectResponse {}

impl TryFrom<http::Response<bytes::Bytes>> for DownloadObjectResponse {
    type Error = Error;

    fn try_from(response: http::Response<bytes::Bytes>) -> Result<Self, Self::Error> {
        let (_parts, body) = response.into_parts();

        Ok(Self { buffer: body })
    }
}

impl std::ops::Deref for DownloadObjectResponse {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.buffer.as_ref()
    }
}

impl io::Read for DownloadObjectResponse {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use bytes::Buf;

        let buf_len = std::cmp::min(self.buffer.len(), buf.len());
        let mut slice = self.buffer.split_to(buf_len);
        slice.copy_to_slice(&mut buf[..buf_len]);

        Ok(buf_len)
    }
}

impl super::Object {
    /// Downloads an object
    ///
    /// Required IAM Permissions: `storage.objects.get`, `storage.objects.getIamPolicy`*
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/json_api/v1/objects/get)
    pub fn download<'a, OID>(
        id: &OID,
        optional: Option<DownloadObjectOptional<'_>>,
    ) -> Result<http::Request<std::io::Empty>, Error>
    where
        OID: ObjectIdentifier<'a> + ?Sized,
    {
        let mut uri = crate::__make_obj_url!(
            "https://www.googleapis.com/storage/v1/b/{}/o/{}?alt=media",
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
