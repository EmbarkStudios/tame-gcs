use failure::{format_err, Error, ResultExt};
use std::{convert::TryInto, sync::Arc};
use tame_oauth::gcp as oauth;

/// Converts a vanilla http::Request into a reqwest::Request
fn convert_request<B>(
    req: http::Request<B>,
    client: &reqwest::Client,
) -> Result<reqwest::Request, Error>
where
    B: std::io::Read + Send + 'static,
{
    let (parts, body) = req.into_parts();

    let uri = parts.uri.to_string();

    let builder = match parts.method {
        http::Method::GET => client.get(&uri),
        http::Method::POST => client.post(&uri),
        http::Method::DELETE => client.delete(&uri),
        http::Method::PUT => client.put(&uri),
        method => unimplemented!("{} not implemented", method),
    };

    struct ProgressRead<B> {
        inner: B,
        pb: indicatif::ProgressBar,
    }

    impl<B> std::io::Read for ProgressRead<B>
    where
        B: std::io::Read + Send + 'static,
    {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let r = self.inner.read(buf)?;
            self.pb.inc(r as u64);
            Ok(r)
        }
    }

    let body = ProgressRead {
        inner: body,
        pb: match tame_gcs::util::get_content_length(&parts.headers) {
            Some(content_len) => indicatif::ProgressBar::new(content_len as u64),
            None => indicatif::ProgressBar::hidden(),
        },
    };

    Ok(builder
        .headers(parts.headers)
        .body(reqwest::Body::new(body))
        .build()?)
}

/// Converts a reqwest::Response into a vanilla http::Response. This currently copies
/// the entire response body into a single buffer with no streaming
fn convert_response(mut res: reqwest::Response) -> Result<http::Response<bytes::Bytes>, Error> {
    use std::io::Read;

    let mut builder = http::Response::builder();

    builder.status(res.status()).version(res.version());

    let headers = builder
        .headers_mut()
        .ok_or_else(|| format_err!("failed to convert response headers"))?;

    headers.extend(
        res.headers()
            .into_iter()
            .map(|(k, v)| (k.clone(), v.clone())),
    );

    let content_len = tame_gcs::util::get_content_length(&headers).unwrap_or_default();
    let mut buffer = bytes::BytesMut::with_capacity(content_len);

    let pb = if content_len > 0 {
        indicatif::ProgressBar::new(content_len as u64)
    } else {
        indicatif::ProgressBar::hidden()
    };

    let mut block = [0u8; 8 * 1024];

    loop {
        let read = res.read(&mut block)?;

        if read == 0 {
            break;
        }

        buffer.extend_from_slice(&block[..read]);
        pb.set_position(buffer.len() as u64);
    }

    pb.finish_and_clear();
    Ok(builder.body(buffer.freeze())?)
}

pub struct RequestContext {
    pub client: reqwest::Client,
    pub cred_path: std::path::PathBuf,
    pub auth: Arc<oauth::ServiceAccountAccess>,
}

/// Executes a GCS request via a reqwest client and returns the parsed response/API error
pub fn execute<B, R>(ctx: &RequestContext, mut req: http::Request<B>) -> Result<R, Error>
where
    R: tame_gcs::ApiResponse<bytes::Bytes>,
    B: std::io::Read + Send + 'static,
{
    // First, get our oauth token, which can mean we have to do an additional
    // request if we've never retrieved one yet, or the one we are using has expired
    let token = match ctx.auth.get_token(&[tame_gcs::Scopes::ReadWrite])? {
        oauth::TokenOrRequest::Token(token) => token,
        oauth::TokenOrRequest::Request {
            request,
            scope_hash,
            ..
        } => {
            let (parts, body) = request.into_parts();
            let read_body = std::io::Cursor::new(body);
            let new_request = http::Request::from_parts(parts, read_body);

            let req = convert_request(new_request, &ctx.client)
                .context("failed to create token request")?;
            let res = ctx
                .client
                .execute(req)
                .context("failed to send token request")?;

            let response = convert_response(res).context("failed to convert token response")?;

            ctx.auth
                .parse_token_response(scope_hash, response)
                .context("failed to parse token response")?
        }
    };

    // Add the authorization token, note that the tame-oauth crate will automatically
    // set the HeaderValue correctly, in the GCP case this is usually "Bearer <token>"
    req.headers_mut()
        .insert(http::header::AUTHORIZATION, token.try_into()?);

    let request = convert_request(req, &ctx.client)?;
    let response = ctx.client.execute(request)?;
    let response = convert_response(response).context("failed to convert response")?;

    Ok(R::try_from_parts(response)?)
}

/// Converts a `gs://<bucket_name>/<object_name>` url into an regular object identifer
pub fn gs_url_to_object_id(url: &url::Url) -> Result<tame_gcs::ObjectId<'_>, Error> {
    match url.scheme() {
        "gs" => {
            let bucket_name = url
                .host_str()
                .ok_or_else(|| format_err!("no bucket specified"))?;
            let object_name = &url.path()[1..];

            Ok(tame_gcs::ObjectId::new(bucket_name, object_name)?)
        }
        scheme => Err(failure::format_err!("invalid url scheme: {}", scheme)),
    }
}
