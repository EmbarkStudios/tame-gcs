use ansi_term::Color;
use failure::{format_err, Error};
use std::path::PathBuf;
use structopt::StructOpt;

mod cat;
#[cfg(feature = "signing")]
mod signurl;
mod util;

#[derive(StructOpt)]
enum Command {
    /// Concatenate object content to stdout
    #[structopt(name = "cat")]
    Cat(cat::Args),
    #[cfg(feature = "signing")]
    #[structopt(name = "signurl")]
    Signurl(signurl::Args),
}

#[derive(StructOpt)]
#[structopt(name = "gsutil")]
struct Opts {
    /// Path to a service account credentials file used to obtain
    /// oauth2 tokens. By default uses GOOGLE_APPLICATION_CREDENTIALS
    /// environment variable.
    #[structopt(short, long, parse(from_os_str))]
    credentials: Option<PathBuf>,
    #[structopt(subcommand)]
    cmd: Command,
}

fn real_main() -> Result<(), Error> {
    let args = Opts::from_args();

    dotenv::dotenv()?;

    let cred_path = args
        .credentials
        .or_else(|| std::env::var_os("GOOGLE_APPLICATION_CREDENTIALS").map(PathBuf::from))
        .ok_or_else(|| format_err!("credentials not specified"))?;

    let client = reqwest::Client::builder().build()?;
    let svc_account_info =
        tame_oauth::gcp::ServiceAccountInfo::deserialize(std::fs::read_to_string(&cred_path)?)?;
    let svc_account_access = tame_oauth::gcp::ServiceAccountAccess::new(svc_account_info)?;

    let ctx = util::RequestContext {
        client,
        cred_path: cred_path,
        auth: std::sync::Arc::new(svc_account_access),
    };

    match args.cmd {
        Command::Cat(args) => cat::cmd(&ctx, args),
        #[cfg(feature = "signing")]
        Command::Signurl(args) => signurl::cmd(&ctx, args),
    }
}

fn main() {
    match real_main() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", Color::Red.paint(format!("{}", e)));
            std::process::exit(1);
        }
    }
}
