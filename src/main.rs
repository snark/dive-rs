extern crate dive;

extern crate getopts;

use std::env;
use std::path::Path;
use std::process;
use getopts::Options;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} path ... [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = Path::new(&args[0])
        .file_name()
        .and_then(|o| o.to_str())
        .unwrap();

    let mut opts = Options::new();
    opts.optflag("a", "all", "Show files whose names begin with a dot (.)");
    opts.optflag("h", "help", "Print this help menu");
    opts.optflag("V", "version", "Print version info and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if matches.opt_present("V") {
        println!("{} version {}", program, VERSION.unwrap_or("unknown"));
        return;
    }

    let config = dive::Config::new(&matches);

    let start_paths = if !matches.free.is_empty() {
        matches.free.clone()
    } else {
        vec![".".to_string()]
    };

    if let Err(e) = dive::run(start_paths, config) {
        println!("ERROR: {}", e);
        process::exit(1);
    }
}
