extern crate dive;

extern crate getopts;

use getopts::Options;
use std::env;
use std::path::Path;
use std::process;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} path ... [options]", program);
    print!("{}", opts.usage(&brief));
}

fn error_out(program: &str, message: &String, show_usage: Option<Options>) {
    println!("{}: {}", program, message);
    show_usage.map(|opts| {
        print!("\n");
        print_usage(program, opts)
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let program_name = match Path::new(&args[0])
        .file_name()
        .and_then(|o| o.to_str()) {
        Some(s) => s,
        None => "dive",
    };

    let mut opts = Options::new();
    opts.optmulti("m", "match", "Filter by basic substring matching", "STRING");
    opts.optmulti("n", "name", "Filter by file name", "PATTERN");
    opts.optmulti("p", "path", "Filter by file path", "PATTERN");
    opts.optmulti("r", "regex", "Filter by regular expression", "REGEX");
    opts.optopt("t",
                "type",
                "Show only files of the specified type",
                "FILETYPE");
    opts.optflag("C", "case-sensitive", "Force case-sensitive matching");
    opts.optflag("a", "all", "Show files whose names begin with a dot (.)");
    opts.optopt("",
                "maxdepth",
                "Descend at most n directories below the starting directories",
                "n");
    opts.optflag("h", "help", "Print this help menu");
    opts.optflag("V", "version", "Print version info and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            error_out(program_name, &f.to_string(), Some(opts));
            process::exit(1);
        }
    };

    if matches.opt_present("h") {
        print_usage(&program_name, opts);
        return;
    }

    if matches.opt_present("V") {
        println!("{} version {}", program_name, VERSION.unwrap_or("unknown"));
        return;
    }

    let config = match dive::config::Config::new(&matches) {
        Ok(c) => c,
        Err(e) => {
            error_out(program_name, &e, None);
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
