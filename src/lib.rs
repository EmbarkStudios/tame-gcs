#![warn(clippy::all)]
#![warn(rust_2018_idioms)]

#[macro_use(Deserialize, Serialize)]
extern crate serde;
#[macro_use(Fail)]
extern crate failure;

#[cfg(feature = "v1")]
mod v1;
#[cfg(feature = "v1")]
pub use crate::v1::*;

pub mod error;
mod response;
pub mod signed_url;
pub mod signing;
pub mod types;
pub mod util;

// Reexport the http crate since everything this crate does
// is put in terms of http request/response
pub use http;

pub use error::Error;
pub use response::{ApiResponse, Response};
pub use types::{BucketName, ObjectId, ObjectName};

/// The [oauth scopes](https://cloud.google.com/storage/docs/authentication)
/// that pertain to Google Cloud Storage.
pub enum Scopes {
    /// Only allows access to read data, including listing buckets.
    ReadOnly,
    /// Allows access to read and change data, but not metadata like IAM policies.
    ReadWrite,
    /// Allows full control over data, including the ability to modify IAM policies.
    FullControl,
    /// View your data across Google Cloud Platform services.
    /// For Cloud Storage, this is the same as `devstorage.read-only`.
    CloudPlatformReadOnly,
    /// View and manage data across all Google Cloud Platform services.
    /// For Cloud Storage, this is the same as `devstorage.full-control`.
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
