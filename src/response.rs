use crate::error::{self, Error};

pub trait ApiResponse<B>: Sized + TryFrom<http::Response<B>, Error = Error>
where
    B: AsRef<[u8]>,
{
    fn try_from_parts(resp: http::response::Response<B>) -> Result<Self, Error> {
        if resp.status().is_success() {
            Self::try_from(resp)
        } else {
            // If we get an error, but with a JSON payload, attempt to deserialize
            // an ApiError from it, otherwise fallback to the simple HttpStatus
            if let Some(ct) = resp
                .headers()
                .get(http::header::CONTENT_TYPE)
                .and_then(|ct| ct.to_str().ok())
            {
                if ct.starts_with("application/json") {
                    if let Ok(api_err) =
                        serde_json::from_slice::<error::ApiError>(resp.body().as_ref())
                    {
                        return Err(Error::Api(api_err));
                    }
                }
            }
            Err(Error::from(resp.status()))
        }
    }
}

pub struct Response<T> {
    body: bytes::BytesMut,
    parts: http::response::Builder,
    content_len: usize,

    _response: std::marker::PhantomData<fn() -> T>,
}

impl<T> Response<T>
where
    T: ApiResponse<bytes::Bytes>,
{
    pub fn new(parts: http::response::Builder) -> Self {
        let body = match parts
            .headers_ref()
            .and_then(crate::util::get_content_length)
        {
            Some(u) => bytes::BytesMut::with_capacity(u),
            None => bytes::BytesMut::new(),
        };

        let content_len = body.capacity();

        Self {
            body,
            parts,
            content_len,
            _response: Default::default(),
        }
    }

    /// Try to get an [`http::Response`]
    pub fn get_response(mut self) -> Result<http::Response<bytes::Bytes>, Error> {
        if self.body.len() >= self.content_len {
            let buf = self.body.split_to(self.content_len);
            let response = self.parts.body(buf.freeze())?;
            Ok(response)
        } else {
            // We need more data, it's possible in a streaming scenario they can
            // call us again with more data
            Err(Error::InsufficientData)
        }
    }

    /// Try to parse all the data buffered so far into a response type.
    pub fn parse(self) -> Result<T, Error> {
        let response = self.get_response()?;
        T::try_from_parts(response)
    }
}

impl<T> std::io::Write for Response<T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.body.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
