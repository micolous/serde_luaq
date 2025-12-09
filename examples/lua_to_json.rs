use clap::{Parser, ValueEnum};
use serde_json::{to_writer, to_writer_pretty};
use serde_luaq::{
    lua_value, return_statement, script, to_json_value, JsonConversionOptions, LuaValue,
};
use std::{fs::File, io::Read, path::PathBuf};

/// Default maximum Lua file size limit.
///
/// 64 MiB is enough for anyone. 🙃
const DEFAULT_SIZE_LIMIT: usize = 64 * 1024 * 1024;
const DEFAULT_MAX_DEPTH: usize = 128;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(ValueEnum, Debug, Copy, Clone)]
enum LuaInputFormat {
    /// Input Lua file is a single Lua expression: `{["foo"] = "bar"}`
    Object,

    /// Input Lua file is a `return` statement: `return {["foo"] = "bar"}`
    Return,

    /// Input Lua file is a Lua script: `foo = "bar"`
    Script,
}

/// Converts a Lua object or script into JSON.
#[derive(Parser, Debug)]
#[command(name = "lua_to_json", version, about, long_about = None, verbatim_doc_comment, rename_all = "snake_case")]
struct Args {
    /// Input Lua filename, will be loaded entirely into memory
    #[arg()]
    input: PathBuf,

    /// Input Lua file format
    #[arg(short, long)]
    format: LuaInputFormat,

    /// Output JSON filename
    #[arg(short, long)]
    output: PathBuf,

    /// Pretty-print JSON output
    #[arg(short, long)]
    pretty: bool,

    /// Don't check that we consumed the entire buffer
    #[arg(short = 'E', long)]
    no_empty_check: bool,

    /// Maximum Lua file size to process
    #[arg(long, default_value_t = DEFAULT_SIZE_LIMIT, id = "BYTES")]
    lua_size_limit: usize,
    
    /// Maximum object depth
    #[arg(long, default_value_t = DEFAULT_MAX_DEPTH, id = "DEPTH")]
    max_depth: usize,

    /// Use lossy string conversion, rather than erroring.
    #[arg(long)]
    lossy_string: bool,
}

fn main() -> Result {
    let args = Args::parse();
    let opts = JsonConversionOptions {
        lossy_string: args.lossy_string,
        ..Default::default()
    };

    let mut f = File::open(args.input)?;
    let size = f.metadata()?.len() as usize;
    if size == 0 {
        panic!("Input file is empty");
    }
    if size > args.lua_size_limit {
        panic!(
            "Maximum file size exceeded ({size} > {})",
            args.lua_size_limit
        );
    }
    let mut buf = Vec::with_capacity(size);
    f.read_to_end(&mut buf)?;

    let lua_value: LuaValue<'_> = match args.format {
        LuaInputFormat::Script => script(&buf, args.max_depth)?.into_iter().collect(),
        LuaInputFormat::Object => lua_value(&buf, args.max_depth)?,
        LuaInputFormat::Return => return_statement(&buf, args.max_depth)?,
    };

    let json_value = to_json_value(lua_value, &opts)?;

    let f = File::options()
        .create_new(true)
        .write(true)
        .open(args.output)?;

    if args.pretty {
        to_writer_pretty(f, &json_value)?;
    } else {
        to_writer(f, &json_value)?;
    }
    Ok(())
}
