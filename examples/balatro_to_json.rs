//! Converts a Balatro config / save file (`.jkr`) to JSON format.
//!
//! These are stored in:
//!
//! * macOS: `~/Library/Application Support/Balatro`.
//! * Windows: `%APPDATA%\Roaming\Balatro`
//!
//! These files are compressed with `deflate`, and contain a Lua value with a single return
//! statement.
use clap::Parser;
use flate2::bufread::DeflateDecoder;
use serde_luaq::{return_statement, to_json_value};
use std::{
    fs::File,
    io::{stdout, BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

const SIZE_LIMIT: usize = 1024 * 1024;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Converts a Balatro .jkr configuration / save game file into JSON format.
#[derive(Parser, Debug)]
#[command(name = "balatro_to_json", version, about, long_about = None, verbatim_doc_comment, rename_all = "snake_case")]
struct Args {
    /// Input Balatro .jkr filename, will be loaded entirely into memory.
    #[arg()]
    input: PathBuf,

    /// Output JSON filename. If not specified, writes to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Pretty-print JSON output.
    #[arg(short, long)]
    pretty: bool,
}

fn main() -> Result {
    let args = Args::parse();

    let f = File::open(args.input)?;
    let size = f.metadata()?.len() as usize;
    if size == 0 {
        panic!("Input file is empty");
    }
    if size > SIZE_LIMIT {
        panic!("Maximum file size exceeded ({size} > {SIZE_LIMIT})");
    }

    let mut f = DeflateDecoder::new(BufReader::new(f));
    let mut buf = Vec::with_capacity(SIZE_LIMIT);
    f.read_to_end(&mut buf)?;

    let map = to_json_value(return_statement(&buf)?, &Default::default())?;

    let mut f: Box<dyn Write> = if let Some(output) = args.output {
        Box::new(BufWriter::new(
            File::options().create_new(true).write(true).open(output)?,
        ))
    } else {
        Box::new(stdout())
    };

    if args.pretty {
        serde_json::to_writer_pretty(&mut f, &map)?;
    } else {
        serde_json::to_writer(&mut f, &map)?;
    }

    f.flush()?;
    Ok(())
}
