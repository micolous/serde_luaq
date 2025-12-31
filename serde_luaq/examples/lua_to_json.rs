use clap::{Parser, ValueEnum};
use serde_json::{to_writer, to_writer_pretty};
use serde_luaq::{
    lua_value, return_statement, script, to_json_value, JsonConversionOptions, LuaValue,
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    fs::File,
    io::{stdout, BufWriter, Read, Write},
    path::PathBuf,
    sync::atomic::{AtomicIsize, Ordering::Relaxed},
};

/// Default maximum Lua file size limit.
///
/// 64 MiB is enough for anyone. 🙃
const DEFAULT_SIZE_LIMIT: usize = 64 * 1024 * 1024;

/// Default maximum table depth (`LUAI_MAXCCALLS`).
const DEFAULT_MAX_DEPTH: u16 = 16;

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
    /// Input Lua filename, will be loaded entirely into memory.
    #[arg()]
    input: PathBuf,

    /// Input Lua file format.
    #[arg(short, long)]
    format: LuaInputFormat,

    /// Output JSON filename; if omitted, writes to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Pretty-print JSON output.
    #[arg(short, long)]
    pretty: bool,

    /// Don't check that we consumed the entire buffer.
    #[arg(short = 'E', long)]
    no_empty_check: bool,

    /// Maximum Lua file size to process.
    #[arg(long, default_value_t = DEFAULT_SIZE_LIMIT, id = "BYTES")]
    lua_size_limit: usize,

    /// Maximum table depth. Increasing this risks the library crashing with a stack overflow.
    #[arg(long, default_value_t = DEFAULT_MAX_DEPTH, id = "DEPTH")]
    max_depth: u16,

    /// Use lossy string conversion, rather than erroring.
    #[arg(long)]
    lossy_string: bool,

    /// Stop once the Lua value has been loaded.
    #[arg(long)]
    no_json: bool,

    /// Convert the Lua value to a JSON object, but don't serialise it.
    #[arg(long)]
    no_output: bool,

    // Print memory usage stats to stderr.
    #[arg(long)]
    memory_stats: bool,
}

// From https://doc.rust-lang.org/std/alloc/struct.System.html
// Modified to use `isize` instead of `usize` so we can track memory usage
// decreases more easily.
struct Counter;
static ALLOCATED: AtomicIsize = AtomicIsize::new(0);
static PEAK: AtomicIsize = AtomicIsize::new(0);
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = unsafe { System.alloc(layout) };
        if !ret.is_null() {
            let r = ALLOCATED.fetch_add(layout.size() as isize, Relaxed);
            PEAK.fetch_max(r, Relaxed);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            System.dealloc(ptr, layout);
        }
        ALLOCATED.fetch_sub(layout.size() as isize, Relaxed);
    }
}

#[global_allocator]
static A: Counter = Counter;

fn main() -> Result {
    let args = Args::parse();
    let start_bytes = ALLOCATED.load(Relaxed);
    if args.memory_stats {
        eprintln!("Initial memory usage: {start_bytes} bytes");
    }

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

    let load_lua_bytes = ALLOCATED.load(Relaxed) - start_bytes;
    if args.memory_stats {
        eprintln!("Reading Lua added: {load_lua_bytes} bytes");
    }

    let lua_value: LuaValue<'_> = match args.format {
        LuaInputFormat::Script => script(&buf, args.max_depth)?.into_iter().collect(),
        LuaInputFormat::Object => lua_value(&buf, args.max_depth)?,
        LuaInputFormat::Return => return_statement(&buf, args.max_depth)?,
    };

    let parse_lua_bytes = ALLOCATED.load(Relaxed) - load_lua_bytes;
    if args.memory_stats {
        let peak = PEAK.load(Relaxed) - load_lua_bytes;
        eprintln!("Parsing Lua added: {parse_lua_bytes} bytes, {peak} peak bytes");
    }

    if args.no_json {
        eprintln!("--no_json set, not converting to JSON!");
        drop(lua_value);
        return Ok(());
    }

    let json_value = to_json_value(lua_value, &opts)?;

    let json_bytes = ALLOCATED.load(Relaxed) - parse_lua_bytes;
    if args.memory_stats {
        eprintln!("Converting to JSON added: {json_bytes} bytes");
    }

    if args.no_output {
        eprintln!("--no_output set, not outputting the result!");
        drop(json_value);
        return Ok(());
    }

    let mut termout = false;
    let f: Box<dyn Write> = if let Some(output) = args.output {
        Box::new(File::options().create_new(true).write(true).open(output)?)
    } else {
        termout = true;
        Box::new(stdout())
    };
    let f = BufWriter::new(f);

    if args.pretty {
        to_writer_pretty(f, &json_value)?;
    } else {
        to_writer(f, &json_value)?;
    }

    if termout {
        println!();
    }

    Ok(())
}
