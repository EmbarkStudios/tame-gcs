use crate::util;
use failure::Error;
use structopt::{clap::arg_enum, StructOpt};
use tame_gcs::{signed_url, signing};

arg_enum! {
    #[derive(Copy, Clone, Debug)]
    pub enum Method {
        Get,
        Post,
        Put,
        Delete,
        Head,
        Options,
        Connect,
        Patch,
        Trace,
        Resumable,
    }
}

fn parse_duration(src: &str) -> Result<std::time::Duration, Error> {
    use std::time::Duration;

    let suffix_pos = src.find(char::is_alphabetic).unwrap_or_else(|| src.len());

    let num: u64 = src[..suffix_pos].parse()?;
    let suffix = if suffix_pos == src.len() {
        "h"
    } else {
        &src[suffix_pos..]
    };

    let duration = match suffix {
        "s" | "S" => Duration::from_secs(num),
        "m" | "M" => Duration::from_secs(num * 60),
        "h" | "H" => Duration::from_secs(num * 60 * 60),
        "d" | "D" => Duration::from_secs(num * 60 * 60 * 24),
        s => return Err(failure::format_err!("unknown duration suffix '{}'", s)),
    };

    Ok(duration)
}

#[derive(StructOpt, Debug)]
pub(crate) struct Args {
    /// The HTTP method to be used with the signed url.
    #[structopt(
        short,
        default_value = "GET",
        raw(possible_values = "&Method::variants()", case_insensitive = "true")
    )]
    method: Method,
    #[structopt(
        short,
        default_value = "1h",
        parse(try_from_str = "parse_duration"),
        long_help = "The duration that ths signed url will be valid for.

Times may be specified with no suffix (default hours), or one of:
* (s)econds
* (m)inutes
* (h)ours
* (d)ays

"
    )]
    duration: std::time::Duration,
    /// The content-type for which the url is valid for, eg. "application/json"
    #[structopt(short)]
    content_type: Option<String>,
    /// The gs:// url
    url: url::Url,
}

pub(crate) fn cmd(ctx: &util::RequestContext, args: Args) -> Result<(), Error> {
    let oid = util::gs_url_to_object_id(&args.url)?;

    let url_signer = signed_url::UrlSigner::with_ring();
    let service_account = signing::ServiceAccount::load_json_file(&ctx.cred_path)?;

    let mut options = signed_url::SignedUrlOptional {
        duration: args.duration,
        ..Default::default()
    };

    if let Some(content_type) = args.content_type {
        options.headers.insert(
            http::header::CONTENT_TYPE,
            http::header::HeaderValue::from_str(&content_type)?,
        );
    }

    options.method = match args.method {
        Method::Get => http::Method::GET,
        Method::Post => http::Method::POST,
        Method::Put => http::Method::PUT,
        Method::Delete => http::Method::DELETE,
        Method::Head => http::Method::HEAD,
        Method::Options => http::Method::OPTIONS,
        Method::Connect => http::Method::CONNECT,
        Method::Patch => http::Method::PATCH,
        Method::Trace => http::Method::TRACE,
        Method::Resumable => {
            options.headers.insert(
                http::header::HeaderName::from_static("x-goog-resumable"),
                http::header::HeaderValue::from_static("start"),
            );
            http::Method::POST
        }
    };

    let signed_url = url_signer.generate(&service_account, &oid, options)?;

    println!("{}", signed_url);

    Ok(())
}
