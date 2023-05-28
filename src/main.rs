use clap::Parser;
use media_org::{process_dir, Params};
use std::env;
use std::io::Error;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
pub struct Args {
    /// Schema to extract
    #[arg(short, long, required = false)]
    pub dir: Option<String>,

    /// Database url to connect to
    #[arg(short, long, required = false)]
    pub out: Option<String>,

    /// Database url to connect to
    #[arg(short, long, required = false)]
    pub copy: bool,

    /// Enable Metadata refresh
    #[arg(short, long, required = false)]
    pub metadata: bool,

    /// Recursive search
    #[arg(short, long, required = false)]
    pub recursive: bool,
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let input_dir = if let Some(dir) = args.dir {
        PathBuf::from(&dir).canonicalize()?
    } else {
        env::current_dir()?.canonicalize()?
    };

    let output_dir = if let Some(dir) = args.out {
        PathBuf::from(&dir)
    } else {
        env::current_dir()?.canonicalize()?
    };

    process_dir(
        input_dir,
        output_dir,
        &Params {
            copy: args.copy,
            metadata: args.metadata,
            recursive: args.recursive,
        },
    )?;

    return Ok(());
}
