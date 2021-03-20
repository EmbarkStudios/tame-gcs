// BEGIN - Embark standard lints v0.3
// do not change or add/remove here, but one can add exceptions after this section
// for more info see: <https://github.com/EmbarkStudios/rust-ecosystem/issues/59>
#![deny(unsafe_code)]
#![warn(
    clippy::all,
    clippy::await_holding_lock,
    clippy::dbg_macro,
    clippy::debug_assert_with_mut_call,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::exit,
    clippy::explicit_into_iter_loop,
    clippy::filter_map_next,
    clippy::fn_params_excessive_bools,
    clippy::if_let_mutex,
    clippy::imprecise_flops,
    clippy::inefficient_to_string,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::map_err_ignore,
    clippy::map_flatten,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::match_same_arms,
    clippy::match_wildcard_for_single_variants,
    clippy::mem_forget,
    clippy::mismatched_target_os,
    clippy::needless_borrow,
    clippy::needless_continue,
    clippy::option_option,
    clippy::pub_enum_variant_names,
    clippy::ref_option_ref,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_add_assign,
    clippy::string_add,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::todo,
    clippy::unimplemented,
    clippy::unnested_or_patterns,
    clippy::unused_self,
    clippy::verbose_file_reads,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms
)]
// END - Embark standard lints v0.3

#[macro_use(Deserialize, Serialize)]
extern crate serde;

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
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
