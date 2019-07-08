#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

#[macro_use(Deserialize)]
extern crate serde;
#[macro_use(Fail)]
extern crate failure;

pub mod error;
pub mod objects;
mod response;
pub mod types;

// Reexport the http crate since everything this crate does
// is put in terms of http request/response
pub use http;

pub use response::{ApiResponse, Response};
pub use types::{BucketName, ObjectName};

/// The oauth scopes that pertain to Google Cloud Storage.
/// See https://cloud.google.com/storage/docs/authentication
pub enum Scopes {
    /// Only allows access to read data, including listing buckets.
    ReadOnly,
    /// Allows access to read and change data, but not metadata like IAM policies.
    ReadWrite,
    /// Allows full control over data, including the ability to modify IAM policies.
    FullControl,
    /// View your data across Google Cloud Platform services.
    /// For Cloud Storage, this is the same as devstorage.read-only.
    CloudPlatformReadOnly,
    /// View and manage data across all Google Cloud Platform services.
    /// For Cloud Storage, this is the same as devstorage.full-control.
    CloudPlatform,
}

impl AsRef<str> for Scopes {
    fn as_ref(&self) -> &str {
        match *self {
            Scopes::ReadOnly => "https://www.googleapis.com/auth/devstorage.read_only",
            Scopes::ReadWrite => "https://www.googleapis.com/auth/devstorage.read_write",
            Scopes::FullControl => "https://www.googleapis.com/auth/devstorage.full_control",
            Scopes::CloudPlatformReadOnly => {
                "https://www.googleapis.com/auth/cloud-platform.read-only"
            }
            Scopes::CloudPlatform => "https://www.googleapis.com/auth/cloud-platform",
        }
    }
}

fn get_content_length(headers: &http::HeaderMap) -> Option<usize> {
    headers.get(http::header::CONTENT_LENGTH).and_then(|h| {
        h.to_str()
            .map_err(|_| ())
            .and_then(|hv| hv.parse::<u64>().map(|l| l as usize).map_err(|_| ()))
            .ok()
    })
}
