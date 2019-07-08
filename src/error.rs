use std::fmt;

#[derive(Fail, Debug, PartialEq)]
pub enum Error {
    #[fail(display = "Expected {}-{} characters, found {}", min, max, len)]
    InvalidCharacterCount { len: usize, min: usize, max: usize },
    #[fail(display = "Expected {}-{} bytes, found {}", min, max, len)]
    InvalidLength { len: usize, min: usize, max: usize },
    #[fail(display = "Character '{}' @ {} is not allowed", _1, _0)]
    InvalidCharacter(usize, char),
    #[fail(display = "Prefix {} is not allowed", _0)]
    InvalidPrefix(&'static str),
    #[fail(display = "Sequence {} is not allowed", _0)]
    InvalidSequence(&'static str),
    #[fail(display = "{}", _0)]
    Http(#[fail(cause)] HttpError),
    #[fail(display = "{}", _0)]
    HttpStatus(#[fail(cause)] HttpStatusError),
    #[fail(display = "An HTTP response didn't have a valid Content-Length")]
    UnknownContentLength,
    #[fail(display = "GCS API error: {}", _0)]
    API(#[fail(cause)] ApiError),
    #[fail(display = "JSON error: {}", _0)]
    Json(#[fail(cause)] JsonError),
    #[fail(display = "Response body doesn't contain enough data")]
    InsufficientData,
}

#[derive(Debug)]
pub struct HttpError(http::Error);

impl PartialEq for HttpError {
    fn eq(&self, _other: &Self) -> bool {
        // I feel really bad about this
        true
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::error::Error::description(self).fmt(f)
    }
}

impl std::error::Error for HttpError {
    fn description(&self) -> &str {
        self.0.description()
    }

    // Return any available cause from the inner error. Note the inner error is
    // not itself the cause.
    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.0.cause()
    }
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Self {
        Error::Http(HttpError(e))
    }
}

#[derive(Debug, Fail)]
pub struct HttpStatusError(http::StatusCode);

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

#[derive(Debug)]
pub struct JsonError(pub(crate) serde_json::Error);

impl PartialEq for JsonError {
    fn eq(&self, other: &Self) -> bool {
        self.0.classify() == other.0.classify()
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::error::Error::description(self).fmt(f)
    }
}

impl std::error::Error for JsonError {
    fn description(&self) -> &str {
        self.0.description()
    }

    // Return any available cause from the inner error. Note the inner error is
    // not itself the cause.
    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.0.cause()
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct ApiErrorInner {
    domain: Option<String>,
    reason: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Fail, PartialEq, Deserialize)]
pub struct ApiError {
    code: u16,
    message: String,
    errors: Vec<ApiErrorInner>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}
