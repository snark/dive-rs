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
    opts.optmulti("m", "match", "Filter by basic substring matching", "STRING");
    opts.optmulti("n", "name", "Filter by file name", "PATTERN");
    opts.optmulti("p", "path", "Filter by file path", "PATTERN");
    opts.optmulti("r", "regex", "Filter by regular expression", "REGEX");
    opts.optflag("a", "all", "Show files whose names begin with a dot (.)");
    opts.optflag("C", "case-sensitive", "Force case-sensitive matching");
    opts.optflag("h", "help", "Print this help menu");
    opts.optflag("V", "version", "Print version info and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}: {}\n", program, f.to_string());
            print_usage(&program, opts);
            process::exit(1);
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    if matches.opt_present("V") {
        println!("{} version {}", program, VERSION.unwrap_or("unknown"));
        return;
    }

    let config = match dive::Config::new(&matches) {
        Ok(c) => c,
        Err(e) => {
            println!("Unable to parse all match rules: {}", e);
            process::exit(1);
        }
    };

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
