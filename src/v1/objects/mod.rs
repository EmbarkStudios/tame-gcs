use crate::common::StorageClass;
use std::collections::BTreeMap;

#[doc(hidden)]
#[macro_export]
macro_rules! __make_obj_url {
    ($url:expr, $id:expr) => {{
        format!(
            $url,
            url::percent_encoding::percent_encode(
                $id.bucket().as_ref(),
                url::percent_encoding::PATH_SEGMENT_ENCODE_SET
            ),
            url::percent_encoding::percent_encode(
                $id.object().as_ref(),
                url::percent_encoding::PATH_SEGMENT_ENCODE_SET
            )
        )
    }};
}

mod delete;
mod download;
mod get;
mod insert;
mod list;

pub use delete::*;
pub use download::*;
pub use get::*;
pub use insert::*;
pub use list::*;

pub struct Object;

/// [Metadata](https://cloud.google.com/storage/docs/json_api/v1/objects#resource)
/// associated with an Object.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// The ID of the object, including the bucket name, object name, and generation number.
    pub id: Option<String>,
    /// The link to this object.
    pub self_link: Option<String>,
    /// The name of the object. Required if not specified by URL parameter. **writable**
    pub name: Option<String>,
    /// The name of the bucket containing this object.
    pub bucket: Option<String>,
    /// The content generation of this object. Used for object versioning.
    #[serde(default, deserialize_with = "from_str_opt")]
    pub generation: Option<i64>,
    /// The version of the metadata for this object at this generation.
    /// Used for preconditions and for detecting changes in metadata.
    /// A metageneration number is only meaningful in the context of a
    /// particular generation of a particular object.
    #[serde(default, deserialize_with = "from_str_opt")]
    pub metageneration: Option<i64>,
    /// `Content-Type` of the object data. If an object is stored without
    /// a `Content-Type`, it is served as `application/octet-stream`. **writable**
    pub content_type: Option<String>,
    /// The creation time of the object in RFC 3339 format.
    pub time_created: Option<chrono::DateTime<chrono::Utc>>,
    /// The modification time of the object metadata in RFC 3339 format.
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Storage class of the object. **writable**
    pub storage_class: Option<StorageClass>,
    /// The time at which the object's storage class was last changed.
    /// When the object is initially created, it will be set to timeCreated.
    pub time_storage_class_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// `Content-Length` of the data in bytes.
    #[serde(default, deserialize_with = "from_str_opt")]
    pub size: Option<u64>,
    /// MD5 hash of the data; encoded using base64. For more information
    /// about using the MD5 hash, see Hashes and ETags: Best Practices. **writable**
    pub md5_hash: Option<String>,
    /// Media download link.
    pub media_link: Option<String>,
    /// `Content-Language` of the object data.
    pub content_language: Option<String>,
    /// CRC32c checksum, as described in RFC 4960, Appendix B; encoded
    /// using base64 in big-endian byte order. For more information about
    /// using the CRC32c checksum, see Hashes and ETags: Best Practices.
    pub crc32c: Option<String>,
    /// HTTP 1.1 Entity tag for the object.
    pub etag: Option<String>,
    /// User-provided metadata, in key/value pairs. **writable**
    pub metadata: Option<BTreeMap<String, String>>,
}

fn from_str_opt<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
    D: serde::de::Deserializer<'de>,
{
    use serde::de::Deserialize;

    let s: &str = Deserialize::deserialize(deserializer)?;
    Ok(Some(T::from_str(&s).map_err(serde::de::Error::custom)?))
}
