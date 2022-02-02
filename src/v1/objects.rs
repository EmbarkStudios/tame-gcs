//! Types and APIs for interacting with GCS [Objects](https://cloud.google.com/storage/docs/json_api/v1/objects)

use crate::common::StorageClass;
use std::collections::BTreeMap;

#[doc(hidden)]
#[macro_export]
macro_rules! __make_obj_url {
    ($url:expr, $id:expr) => {{
        format!(
            $url,
            percent_encoding::percent_encode($id.bucket().as_ref(), crate::util::PATH_ENCODE_SET),
            percent_encoding::percent_encode($id.object().as_ref(), crate::util::PATH_ENCODE_SET)
        )
    }};
}

mod delete;
mod download;
mod get;
mod insert;
mod list;
mod patch;
mod rewrite;

pub use delete::*;
pub use download::*;
pub use get::*;
pub use insert::*;
pub use list::*;
pub use patch::*;
pub use rewrite::*;

pub type Timestamp = time::OffsetDateTime;

/// Helper struct used to collate all of the operations available for
/// [Objects](https://cloud.google.com/storage/docs/json_api/v1/objects)
pub struct Object;

/// [Metadata](https://cloud.google.com/storage/docs/json_api/v1/objects#resource)
/// associated with an Object.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// The ID of the object, including the bucket name, object name, and generation number.
    #[serde(skip_serializing)]
    pub id: Option<String>,
    /// The link to this object.
    #[serde(skip_serializing)]
    pub self_link: Option<String>,
    /// The name of the object. Required if not specified by URL parameter. **writable**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The name of the bucket containing this object.
    #[serde(skip_serializing)]
    pub bucket: Option<String>,
    /// The content generation of this object. Used for object versioning.
    #[serde(default, skip_serializing, deserialize_with = "from_str_opt")]
    pub generation: Option<i64>,
    /// The version of the metadata for this object at this generation.
    /// Used for preconditions and for detecting changes in metadata.
    /// A metageneration number is only meaningful in the context of a
    /// particular generation of a particular object.
    #[serde(default, skip_serializing, deserialize_with = "from_str_opt")]
    pub metageneration: Option<i64>,
    /// `Content-Type` of the object data. If an object is stored without
    /// a `Content-Type`, it is served as `application/octet-stream`. **writable**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// `Content-Disposition` of the object data. **writable**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_disposition: Option<String>,
    /// `Content-Encoding` of the object data. **writable**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,
    /// The creation time of the object in RFC 3339 format.
    #[serde(default, skip_serializing, deserialize_with = "timestamp_rfc3339_opt")]
    pub time_created: Option<Timestamp>,
    /// The modification time of the object metadata in RFC 3339 format.
    #[serde(default, skip_serializing, deserialize_with = "timestamp_rfc3339_opt")]
    pub updated: Option<Timestamp>,
    /// Storage class of the object. **writable**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<StorageClass>,
    /// The time at which the object's storage class was last changed.
    /// When the object is initially created, it will be set to timeCreated.
    #[serde(default, skip_serializing, deserialize_with = "timestamp_rfc3339_opt")]
    pub time_storage_class_updated: Option<Timestamp>,
    /// `Content-Length` of the data in bytes.
    #[serde(default, skip_serializing, deserialize_with = "from_str_opt")]
    pub size: Option<u64>,
    /// MD5 hash of the data; encoded using base64. For more information
    /// about using the MD5 hash, see Hashes and ETags: Best Practices. **writable**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5_hash: Option<String>,
    /// Media download link.
    #[serde(skip_serializing)]
    pub media_link: Option<String>,
    /// `Content-Language` of the object data. **writable**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_language: Option<String>,
    /// CRC32c checksum, as described in RFC 4960, Appendix B; encoded
    /// using base64 in big-endian byte order. For more information about
    /// using the CRC32c checksum, see Hashes and ETags: Best Practices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crc32c: Option<String>,
    /// HTTP 1.1 Entity tag for the object.
    #[serde(skip_serializing)]
    pub etag: Option<String>,
    /// User-provided metadata, in key/value pairs. **writable**
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, String>>,
}

use serde::de::Deserialize;

fn from_str_opt<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
    D: serde::de::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    T::from_str(s).map_err(serde::de::Error::custom).map(Some)
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
    D: serde::de::Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    T::from_str(s).map_err(serde::de::Error::custom)
}

fn timestamp_rfc3339_opt<'de, D>(deserializer: D) -> Result<Option<Timestamp>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let ts_str: &str = Deserialize::deserialize(deserializer)?;
    Timestamp::parse(ts_str, &time::format_description::well_known::Rfc3339)
        .map_err(serde::de::Error::custom)
        .map(Some)
}
