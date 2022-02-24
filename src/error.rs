//! Error facilities

use std::fmt;

/// Core error type for all errors possible from tame-gcs
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("Expected {min}-{max} characters, found {len}")]
    InvalidCharacterCount { len: usize, min: usize, max: usize },
    #[error("Expected {min}-{max} bytes, found {len}")]
    InvalidLength { len: usize, min: usize, max: usize },
    #[error("Character '{1}' @ {0} is not allowed")]
    InvalidCharacter(usize, char),
    #[error("Prefix {0} is not allowed")]
    InvalidPrefix(&'static str),
    #[error("Sequence {0} is not allowed")]
    InvalidSequence(&'static str),
    #[error("Failed to parse URI")]
    InvalidUri(UriError),
    #[error("HTTP error")]
    Http(#[source] HttpError),
    #[error("HTTP status")]
    HttpStatus(#[source] HttpStatusError),
    #[error("An HTTP response didn't have a valid {0}")]
    UnknownHeader(http::header::HeaderName),
    #[error("GCS API error")]
    Api(#[source] ApiError),
    #[error("JSON error")]
    Json(#[source] JsonError),
    #[error("Response body doesn't contain enough data")]
    InsufficientData,
    #[error("Key rejected: {0}")]
    KeyRejected(String),
    #[error("An error occurred during signing")]
    Signing,
    #[error("An expiration duration was too long: requested = {requested}, max = {max}")]
    TooLongExpiration { requested: u64, max: u64 },
    #[error("Failed to parse url")]
    UrlParse(#[source] url::ParseError),
    #[error("Unable to stringize or parse header value '{0:?}'")]
    OpaqueHeaderValue(http::header::HeaderValue),
    #[error("I/O error occurred")]
    Io(#[source] IoError),
    #[error("Unable to decode base64")]
    Base64Decode(#[source] base64::DecodeError),
    #[error("Unable to encode url")]
    UrlEncode(#[source] serde_urlencoded::ser::Error),
}

#[derive(Debug, thiserror::Error)]
pub struct HttpError(#[source] pub http::Error);

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for HttpError {
    fn eq(&self, _other: &Self) -> bool {
        // I feel really bad about this
        true
    }
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Self {
        Error::Http(HttpError(e))
    }
}

#[derive(Debug, thiserror::Error)]
pub struct HttpStatusError(pub http::StatusCode);

impl PartialEq for HttpStatusError {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl fmt::Display for HttpStatusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<http::StatusCode> for Error {
    fn from(e: http::StatusCode) -> Self {
        Error::HttpStatus(HttpStatusError(e))
    }
}

#[derive(Debug, thiserror::Error)]
pub struct IoError(#[source] pub std::io::Error);

impl PartialEq for IoError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(IoError(e))
    }
}

#[derive(Debug, thiserror::Error)]
pub struct JsonError(#[source] pub serde_json::Error);

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for JsonError {
    fn eq(&self, other: &Self) -> bool {
        self.0.classify() == other.0.classify()
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Json(JsonError(e))
    }
}

impl From<serde_urlencoded::ser::Error> for Error {
    fn from(e: serde_urlencoded::ser::Error) -> Self {
        Error::UrlEncode(e)
    }
}

#[derive(Debug, thiserror::Error)]
pub struct UriError(#[source] http::uri::InvalidUri);

impl fmt::Display for UriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for UriError {
    fn eq(&self, other: &Self) -> bool {
        // This is **TERRIBLE** but all of the error details are unnecessarily
        // private and it doesn't implement PartialEq ARGH
        self.0.to_string() == other.0.to_string()
    }
}

impl From<http::uri::InvalidUri> for Error {
    fn from(e: http::uri::InvalidUri) -> Self {
        Error::InvalidUri(UriError(e))
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct ApiErrorInner {
    pub domain: Option<String>,
    pub reason: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, thiserror::Error, PartialEq, Deserialize)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub errors: Vec<ApiErrorInner>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

#[cfg(feature = "signing")]
impl From<ring::error::KeyRejected> for Error {
    fn from(re: ring::error::KeyRejected) -> Self {
        Error::KeyRejected(format!("{}", re))
    }
}

#[cfg(feature = "signing")]
impl From<ring::error::Unspecified> for Error {
    fn from(_re: ring::error::Unspecified) -> Self {
        Error::Signing
    }
}
