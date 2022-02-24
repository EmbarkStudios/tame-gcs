use crate::{
    common::{Conditionals, Projection, StandardQueryParameters},
    error::Error,
    response::ApiResponse,
    types::ObjectIdentifier,
};

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObjectOptional<'a> {
    #[serde(flatten)]
    pub standard_params: StandardQueryParameters<'a>,
    /// Resource name of the Cloud KMS key that will be used to encrypt the
    /// object. The Cloud KMS key must be located in same location as the object.
    ///
    /// If the parameter is not specified, the method uses the destination
    /// bucket's default encryption key, if any, or the Google-managed encryption
    /// key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_kms_key_name: Option<String>,
    /// Apply a predefined set of access controls to the destination object.
    ///
    /// Acceptable values are:
    ///
    /// * authenticatedRead: Object owner gets OWNER access, and
    /// allAuthenticatedUsers get READER access.
    /// * bucketOwnerFullControl: Object owner gets OWNER access, and project
    /// team owners get OWNER access.
    /// * bucketOwnerRead: Object owner gets OWNER access, and project team
    /// owners get READER access.
    /// * private: Object owner gets OWNER access.
    /// * projectPrivate: Object owner gets OWNER access, and project team
    /// members get access according to their roles.
    /// * publicRead: Object owner gets OWNER access, and allUsers get READER access.
    ///
    /// If iamConfiguration.uniformBucketLevelAccess.enabled is set to true,
    /// requests that include this parameter fail with a 400 Bad Request response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_predefined_acl: Option<String>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub destination_conditionals: Option<Conditionals>,
    /// Makes the operation conditional on whether the source object's
    /// generation matches the given value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_source_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's
    /// generation does not match the given value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_source_generation_not_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// metageneration matches the given value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_source_metageneration_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// metageneration does not match the given value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_source_metageneration_not_match: Option<i64>,
    /// The maximum number of bytes that will be rewritten per rewrite request.
    /// Most callers shouldn't need to specify this parameter - it is primarily
    /// in place to support testing. If specified the value must be an integral
    /// multiple of 1 MiB (1048576). Also, this only applies to requests where
    /// the source and destination span locations and/or storage classes.
    /// Finally, this value must not change across rewrite calls else you'll get
    /// an error that the rewriteToken is invalid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bytes_rewritten_per_call: Option<i64>,
    /// Set of properties to return. Defaults to `noAcl`, unless the object
    /// resource specifies the acl property, when it defaults to full.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<Projection>,
    /// If present, selects a specific revision of the source object (as opposed
    /// to the latest version, the default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_generation: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObjectResponse {
    /// The number of bytes that have been rewritten thusfar
    #[serde(deserialize_with = "crate::objects::from_str")]
    pub total_bytes_rewritten: u64,
    /// The total size of the original source object
    #[serde(deserialize_with = "crate::objects::from_str")]
    pub object_size: u64,
    /// Indicates if the rewrite is finished or not
    pub done: bool,
    /// If done is false, this will be Some() and it must be specified in each
    /// additional rewrite call until done is true
    pub rewrite_token: Option<String>,
    #[serde(rename = "resource")]
    pub metadata: Option<super::Metadata>,
}

impl ApiResponse<bytes::Bytes> for RewriteObjectResponse {}

impl<B> TryFrom<http::Response<B>> for RewriteObjectResponse
where
    B: AsRef<[u8]>,
{
    type Error = Error;

    fn try_from(response: http::Response<B>) -> Result<Self, Self::Error> {
        let (_parts, body) = response.into_parts();
        Ok(serde_json::from_slice(body.as_ref())?)
    }
}

impl super::Object {
    /// Rewrites a source object to a destination object. Optionally overrides metadata.
    ///
    /// Required IAM Permissions:
    /// * `storage.objects.create` (for the destination bucket)
    /// * `storage.objects.delete` (for the destination bucket)
    /// * `storage.objects.get` (for the source bucket)
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/json_api/v1/objects/rewrite)
    pub fn rewrite<'a, OID>(
        source: &OID,
        destination: &OID,
        rewrite_token: Option<String>,
        metadata: Option<&super::Metadata>,
        optional: Option<RewriteObjectOptional<'_>>,
    ) -> Result<http::Request<std::io::Cursor<Vec<u8>>>, Error>
    where
        OID: ObjectIdentifier<'a> + ?Sized,
    {
        let mut uri = format!(
            "https://storage.googleapis.com/storage/v1/b/{}/o/{}/rewriteTo/b/{}/o/{}",
            percent_encoding::percent_encode(
                source.bucket().as_ref(),
                crate::util::PATH_ENCODE_SET
            ),
            percent_encoding::percent_encode(
                source.object().as_ref(),
                crate::util::PATH_ENCODE_SET
            ),
            percent_encoding::percent_encode(
                destination.bucket().as_ref(),
                crate::util::PATH_ENCODE_SET
            ),
            percent_encoding::percent_encode(
                destination.object().as_ref(),
                crate::util::PATH_ENCODE_SET
            )
        );

        let query = optional.unwrap_or_default();
        let query_params = serde_urlencoded::to_string(query)?;
        if !query_params.is_empty() || rewrite_token.is_some() {
            uri.push('?');

            if let Some(rt) = rewrite_token {
                uri.push_str("rewriteToken=");
                uri.push_str(&rt);

                if !query_params.is_empty() {
                    uri.push('&');
                }
            }

            if !query_params.is_empty() {
                uri.push_str(&query_params);
            }
        }

        let mut req_builder = http::Request::builder();

        let body = match metadata {
            Some(metadata) => {
                let md = serde_json::to_vec(&metadata)?;
                let len = md.len();

                req_builder = req_builder
                    .header("content-type", "application/json")
                    .header("content-length", len);

                std::io::Cursor::new(md)
            }
            None => std::io::Cursor::new(Vec::new()),
        };

        Ok(req_builder.method("POST").uri(uri).body(body)?)
    }
}
