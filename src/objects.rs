use crate::{
    error::{self, Error},
    response::ApiResponse,
    types::{BucketName, ObjectName},
};
use std::{collections::BTreeMap, convert::TryFrom, io};
use url::percent_encoding as perc_enc;

pub struct Object;

impl Object {
    // https://cloud.google.com/storage/docs/json_api/v1/objects/insert
    pub fn insert<B>(
        bucket: &BucketName<'_>,
        name: &ObjectName<'_>,
        content: B,
        optional: InsertObjectOptional<'_>,
    ) -> Result<http::Request<B>, Error> {
        let uri = format!(
            "https://www.googleapis.com/upload/storage/v1/b/{}/o?name={}&uploadType=media",
            perc_enc::percent_encode(bucket.as_ref(), perc_enc::PATH_SEGMENT_ENCODE_SET),
            perc_enc::percent_encode(name.as_ref(), perc_enc::QUERY_ENCODE_SET)
        );

        let mut req_builder = http::Request::builder();

        if let Some(ct) = optional.content_type {
            req_builder.header(
                http::header::CONTENT_TYPE,
                http::header::HeaderValue::from_str(ct).map_err(http::Error::from)?,
            );
        }

        Ok(req_builder.method("POST").uri(uri).body(content)?)
    }

    // Downloads the object
    // https://cloud.google.com/storage/docs/json_api/v1/objects/get
    pub fn download(
        bucket: &BucketName<'_>,
        name: &ObjectName<'_>,
        optional: DownloadObjectOptional,
    ) -> Result<http::Request<std::io::Empty>, Error> {
        let uri = format!(
            "https://www.googleapis.com/storage/v1/b/{}/o/{}?alt=media",
            perc_enc::percent_encode(bucket.as_ref(), perc_enc::PATH_SEGMENT_ENCODE_SET),
            perc_enc::percent_encode(name.as_ref(), perc_enc::PATH_SEGMENT_ENCODE_SET)
        );

        let mut query_pairs = url::form_urlencoded::Serializer::new(uri);

        if let Some(generation) = optional.generation {
            query_pairs.append_pair("generation", &generation.to_string());
        }

        let uri = query_pairs.finish();

        let mut req_builder = http::Request::builder();

        Ok(req_builder.method("GET").uri(uri).body(std::io::empty())?)
    }

    // Gets the object's metadata
    // https://cloud.google.com/storage/docs/json_api/v1/objects/get
    pub fn get(
        bucket: &BucketName<'_>,
        name: &ObjectName<'_>,
        optional: GetObjectOptional,
    ) -> Result<http::Request<std::io::Empty>, Error> {
        let uri = format!(
            "https://www.googleapis.com/storage/v1/b/{}/o/{}",
            perc_enc::percent_encode(bucket.as_ref(), perc_enc::PATH_SEGMENT_ENCODE_SET),
            perc_enc::percent_encode(name.as_ref(), perc_enc::PATH_SEGMENT_ENCODE_SET)
        );

        let mut query_pairs = url::form_urlencoded::Serializer::new(uri);

        if let Some(generation) = optional.generation {
            query_pairs.append_pair("generation", &generation.to_string());
        }

        let uri = query_pairs.finish();

        let mut req_builder = http::Request::builder();

        Ok(req_builder.method("GET").uri(uri).body(std::io::empty())?)
    }

    // https://cloud.google.com/storage/docs/json_api/v1/objects/delete
    pub fn delete(
        bucket: &BucketName<'_>,
        name: &ObjectName<'_>,
        optional: DeleteObjectOptional,
    ) -> Result<http::Request<std::io::Empty>, Error> {
        let uri = format!(
            "https://www.googleapis.com/storage/v1/b/{}/o/{}",
            perc_enc::percent_encode(bucket.as_ref(), perc_enc::PATH_SEGMENT_ENCODE_SET),
            perc_enc::percent_encode(name.as_ref(), perc_enc::PATH_SEGMENT_ENCODE_SET)
        );

        let mut query_pairs = url::form_urlencoded::Serializer::new(uri);

        if let Some(generation) = optional.generation {
            query_pairs.append_pair("generation", &generation.to_string());
        }

        let uri = query_pairs.finish();

        let mut req_builder = http::Request::builder();

        Ok(req_builder
            .method("DELETE")
            .uri(uri)
            .body(std::io::empty())?)
    }
}

#[derive(Default)]
pub struct InsertObjectOptional<'a> {
    pub content_type: Option<&'a str>,
}

pub struct InsertObjectResponse {
    _buffer: bytes::Bytes,
}

impl ApiResponse for InsertObjectResponse {}

impl TryFrom<bytes::Bytes> for InsertObjectResponse {
    type Error = Error;

    fn try_from(b: bytes::Bytes) -> Result<Self, Self::Error> {
        Ok(Self { _buffer: b })
    }
}

#[derive(Default)]
pub struct DownloadObjectOptional {
    pub generation: Option<i64>,
}

pub struct DownloadObjectResponse {
    buffer: bytes::Bytes,
}

impl DownloadObjectResponse {
    pub fn consume(self) -> bytes::Bytes {
        self.buffer
    }
}

impl ApiResponse for DownloadObjectResponse {}

impl TryFrom<bytes::Bytes> for DownloadObjectResponse {
    type Error = Error;

    fn try_from(b: bytes::Bytes) -> Result<Self, Self::Error> {
        Ok(Self { buffer: b })
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
        use bytes::{Buf, IntoBuf};

        let max = std::cmp::max(self.buffer.len(), buf.len());
        let mut slice = self.buffer.split_to(max).into_buf();
        slice.copy_to_slice(buf);

        Ok(max)
    }
}

#[derive(Default)]
pub struct GetObjectOptional {
    generation: Option<i64>,
}

pub struct GetObjectResponse {
    buffer: bytes::Bytes,
}

impl<'a> GetObjectResponse {
    pub fn metadata(&'a self) -> Result<ObjectMetadata<'a>, Error> {
        serde_json::from_slice(&self.buffer).map_err(|e| Error::Json(error::JsonError(e)))
    }
}

impl ApiResponse for GetObjectResponse {}

impl TryFrom<bytes::Bytes> for GetObjectResponse {
    type Error = Error;

    fn try_from(buffer: bytes::Bytes) -> Result<Self, Self::Error> {
        // Maybe could try Pin out
        let res = Self { buffer };

        let _ = res.metadata()?;

        Ok(res)
    }
}

#[derive(Default)]
pub struct DeleteObjectOptional {
    pub generation: Option<i64>,
}

pub struct DeleteObjectResponse {
    _buffer: bytes::Bytes,
}

impl ApiResponse for DeleteObjectResponse {}

impl TryFrom<bytes::Bytes> for DeleteObjectResponse {
    type Error = Error;

    fn try_from(b: bytes::Bytes) -> Result<Self, Self::Error> {
        Ok(Self { _buffer: b })
    }
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

/// Metadata associated with an Object, see
/// https://cloud.google.com/storage/docs/json_api/v1/objects#resource.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectMetadata<'a> {
    pub kind: Option<&'a str>,
    pub id: Option<&'a str>,
    pub self_link: Option<&'a str>,
    pub name: Option<&'a str>,
    pub bucket: Option<&'a str>,
    #[serde(default, deserialize_with = "from_str_opt")]
    pub generation: Option<i64>,
    #[serde(default, deserialize_with = "from_str_opt")]
    pub metageneration: Option<i64>,
    pub content_type: Option<&'a str>,
    pub time_created: Option<chrono::DateTime<chrono::Utc>>,
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
    pub storage_class: Option<&'a str>,
    pub time_storage_class_updated: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, deserialize_with = "from_str_opt")]
    pub size: Option<u64>,
    pub md5_hash: Option<&'a str>,
    pub media_link: Option<&'a str>,
    pub content_language: Option<&'a str>,
    pub crc32c: Option<&'a str>,
    pub etag: Option<&'a str>,
    pub metadata: Option<BTreeMap<&'a str, &'a str>>,
}
