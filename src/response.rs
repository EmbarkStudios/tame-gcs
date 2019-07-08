use crate::error::{self, Error};
use std::convert::TryFrom;

pub trait ApiResponse: Sized + TryFrom<bytes::Bytes, Error = Error> {
    fn try_from_parts(
        parts: &http::response::Parts,
        buffer: &mut bytes::Bytes,
    ) -> Result<Self, Error> {
        // For now just assume we get all the things unless otherwise specified
        let content_len = crate::get_content_length(&parts.headers).unwrap_or_else(|| buffer.len());
        if buffer.len() >= content_len {
            let buffer = buffer.split_to(content_len);

            if parts.status.is_success() {
                Self::try_from(buffer)
            } else {
                // If we get an error, but with a JSON payload, attempt to deserialize
                // an ApiError from it, otherwise fallback to the simple HttpStatus
                if let Some(ct) = parts
                    .headers
                    .get(http::header::CONTENT_TYPE)
                    .and_then(|ct| ct.to_str().ok())
                {
                    if ct.starts_with("application/json") {
                        if let Ok(api_err) = serde_json::from_slice::<error::ApiError>(&buffer) {
                            return Err(Error::API(api_err));
                        }
                    }
                }

                Err(Error::from(parts.status))
            }
        } else {
            // We need more data, it's possible in a streaming scenario they can
            // call us again with more data
            Err(Error::InsufficientData)
        }
    }
}

pub struct Response {
    body: bytes::BytesMut,
    parts: http::response::Builder,
}

impl std::io::Write for Response {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.body.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Response {
    pub fn new(parts: http::response::Builder) -> Self {
        let body = match parts
            .headers_ref()
            .and_then(|hm| crate::get_content_length(hm))
        {
            Some(u) => bytes::BytesMut::with_capacity(u),
            None => bytes::BytesMut::new(),
        };

        Self { body, parts }
    }

    pub fn finish(mut self) -> Result<http::Response<bytes::Bytes>, Error> {
        Ok(self.parts.body(self.body.freeze())?)
    }
}
