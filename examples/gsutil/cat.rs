use crate::util;
use failure::Error;
use structopt::StructOpt;
use tame_gcs::objects::Object;

#[derive(StructOpt, Debug)]
pub(crate) struct Args {
    /// Causes gsutil to output just the specified byte range of the
    /// object. Ranges are can be of these forms:
    ///
    ///     start-end (e.g., -r 256-5939)
    ///
    ///     start-    (e.g., -r 256-)
    ///
    ///     -numbytes (e.g., -r -5)
    ///
    /// where offsets start at 0, start-end means to return bytes start
    /// through end (inclusive), start- means to return bytes start
    /// through the end of the object, and -numbytes means to return the
    /// last numbytes of the object.
    #[structopt(short)]
    range: Option<String>,
    /// The gs url to the object
    url: url::Url,
}

pub(crate) fn cmd(ctx: &util::RequestContext, args: Args) -> Result<(), Error> {
    let oid = util::gs_url_to_object_id(&args.url)?;

    let mut download_req = Object::download(&oid, None)?;

    if let Some(range) = args.range {
        download_req.headers_mut().insert(
            http::header::RANGE,
            http::header::HeaderValue::from_str(&format!("bytes={}", range))?,
        );
    }

    let mut response: tame_gcs::objects::DownloadObjectResponse = util::execute(ctx, download_req)?;

    std::io::copy(&mut response, &mut std::io::stdout())?;

    Ok(())
}
