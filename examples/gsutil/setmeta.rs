use crate::util;
use anyhow::Error;
use structopt::StructOpt;
use tame_gcs::objects::Object;

#[derive(StructOpt, Debug)]
pub(crate) struct Args {
    /// A valid JSON payload for the metadata to set
    json: String,
    /// The gs:// url to the object to set metadata for
    url: url::Url,
}

pub(crate) async fn cmd(ctx: &util::RequestContext, args: Args) -> Result<(), Error> {
    let oid = util::gs_url_to_object_id(&args.url)?;

    let md: tame_gcs::objects::Metadata = serde_json::from_str(&args.json)?;

    let set_req = Object::patch(
        &(
            oid.bucket(),
            oid.object()
                .ok_or_else(|| anyhow::anyhow!("invalid object name specified"))?,
        ),
        &md,
        None,
    )?;

    let get_res: tame_gcs::objects::PatchObjectResponse = util::execute(ctx, set_req).await?;

    let md = get_res.metadata;

    // Print out the information the same way gsutil does, except with RFC-2822 date formatting
    println!("{}", ansi_term::Color::Cyan.paint(args.url.as_str()));
    println!(
        "    Creation time:\t{}",
        md.time_created.expect("time_created").to_rfc2822()
    );
    println!(
        "    Update time:\t{}",
        md.updated.expect("updated").to_rfc2822()
    );
    println!(
        "    Storage class:\t{}",
        md.storage_class.expect("storage_class")
    );
    println!("    Content-Length:\t{}", md.size.expect("size"));
    println!(
        "    Content-Type:\t{}",
        md.content_type.as_deref().unwrap_or("None")
    );

    if let Some(md) = &md.metadata {
        for (k, v) in md {
            println!("        {}:\t\t{}", k, v);
        }
    }

    println!("    Hash (crc32c):\t{}", md.crc32c.expect("crc32c"));
    println!("    Hash (md5):\t\t{}", md.md5_hash.expect("md5_hash"));
    println!("    ETag:\t\t{}", md.etag.expect("etag"));
    println!("    Generation:\t\t{}", md.generation.expect("generation"));
    println!(
        "    Metageneration:\t{}",
        md.metageneration.expect("metageneration")
    );

    Ok(())
}
