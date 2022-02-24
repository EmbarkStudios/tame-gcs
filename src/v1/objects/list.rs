use crate::{
    common::{Projection, StandardQueryParameters},
    error::Error,
    response::ApiResponse,
    types::BucketName,
};

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListOptional<'a> {
    #[serde(flatten)]
    pub standard_params: StandardQueryParameters<'a>,
    /// Returns results in a directory-like mode. items will contain
    /// only objects whose names, aside from the prefix, do not contain
    /// delimiter. Objects whose names, aside from the prefix, contain
    /// delimiter will have their name, truncated after the delimiter,
    /// returned in prefixes. Duplicate prefixes are omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delimiter: Option<&'a str>,
    /// If true, objects that end in exactly one instance of delimiter
    /// will have their metadata included in items in addition to prefixes.
    #[serde(skip_serializing_if = "crate::util::if_false")]
    pub include_trailing_delimiter: bool,
    /// Maximum number of items plus prefixes to return in a single page
    /// of responses. As duplicate prefixes are omitted, fewer total
    /// results may be returned than requested. The service will use
    /// this parameter or 1,000 items, whichever is smaller.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
    /// A previously-returned page token representing part of the larger
    /// set of results to view.
    ///
    /// The pageToken is an encoded field that marks the name and generation
    /// of the last object in the returned list. In a subsequent request
    /// using the pageToken, items that come after the pageToken are shown
    /// (up to maxResults).
    ///
    /// If you start a listing and then create an object in the bucket before
    ///  using a pageToken to continue listing, you will not see the new
    /// object in subsequent listing results if it is in part of the object
    /// namespace already listed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<&'a str>,
    /// Filter results to objects whose names begin with this prefix.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<Projection>,
    /// The project to be billed for this request.
    /// Required for Requester Pays buckets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_project: Option<&'a str>,
    /// If true, lists all versions of an object as distinct results.
    /// The default is false. For more information, see Object Versioning.
    #[serde(skip_serializing_if = "crate::util::if_false")]
    pub versions: bool,
}

pub struct ListResponse {
    /// The list of objects matching the query
    pub objects: Vec<super::Metadata>,
    /// The list of prefixes of objects matching-but-not-listed up to
    /// and including the requested delimiter.
    pub prefixes: Vec<String>,
    /// The continuation token, included only if there are more items to return.
    /// Provide this value as the page_token of a subsequent request in order
    /// to return the next page of results.
    pub page_token: Option<String>,
}

impl ApiResponse<&[u8]> for ListResponse {}
impl ApiResponse<bytes::Bytes> for ListResponse {}

impl<B> TryFrom<http::Response<B>> for ListResponse
where
    B: AsRef<[u8]>,
{
    type Error = Error;

    fn try_from(response: http::Response<B>) -> Result<Self, Self::Error> {
        let (_parts, body) = response.into_parts();

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RawListResponse {
            next_page_token: Option<String>,
            #[serde(default)]
            prefixes: Vec<String>,
            // This field won't be present if the list doesn't actually
            // return any items
            #[serde(default)]
            items: Vec<super::Metadata>,
        }

        let res: RawListResponse = serde_json::from_slice(body.as_ref())?;

        Ok(Self {
            objects: res.items,
            prefixes: res.prefixes,
            page_token: res.next_page_token,
        })
    }
}

impl super::Object {
    /// Retrieves a list of objects matching the criteria.
    ///
    /// Required IAM Permissions: `storage.objects.list`, `storage.objects.getIamPolicy`*
    ///
    /// [Complete API Documentation](https://cloud.google.com/storage/docs/json_api/v1/objects/list)
    pub fn list(
        bucket: &BucketName<'_>,
        optional: Option<ListOptional<'_>>,
    ) -> Result<http::Request<std::io::Empty>, Error> {
        let mut uri = format!("https://www.googleapis.com/storage/v1/b/{}/o", bucket);

        let query = optional.unwrap_or_default();
        let query_params = serde_urlencoded::to_string(query)?;
        if !query_params.is_empty() {
            uri.push('?');
            uri.push_str(&query_params);
        }

        let req_builder = http::Request::builder();

        Ok(req_builder.method("GET").uri(uri).body(std::io::empty())?)
    }
}
