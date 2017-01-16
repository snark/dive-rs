extern crate getopts;
extern crate glob;
extern crate ignore;

use std::error::Error;
use std::path::Path;

use glob::Pattern;
use ignore::{WalkBuilder, DirEntry};
use ignore::overrides::OverrideBuilder;

pub struct Config {
    pub all: bool,
    pub name: Option<Pattern>,
}

impl Config {
    pub fn new(matches: &getopts::Matches) -> Config {
        let name = match matches.opt_str("name") {
            Some(g) => Pattern::new(&g).ok(),
            _ => None,
        };
        Config {
            all: matches.opt_present("all"),
            name: name,
        }
    }
}

fn handle_entry(entry: DirEntry, config: &Config) {
    let ref config_name = config.name;
    let m = match config_name.as_ref() {
        Some(p) => {
            let n = entry.file_name().to_str();
            p.matches(n.unwrap())
        }
        None => true,
    };
    if m {
        println!("{}", entry.path().display());
    };
}

pub fn run(start_paths: Vec<std::string::String>, config: Config) -> Result<(), Box<Error>> {
    for path_string in start_paths {
        let exists;
        {
            let path = Path::new(&path_string);
            exists = path.exists();
        }
        if !exists {
            println!("{}: No such file or directory", path_string);
        } else {
            let mut builder = WalkBuilder::new(path_string);
            if config.all {
                builder.hidden(false);
            };
            // Overrides
            if config.all {
                let mut override_builder = OverrideBuilder::new("/");
                override_builder.add(&"!.git/").unwrap();
                builder.overrides(override_builder.build().unwrap());
            }
            let walker = builder.build();
            // Currently errors are silently swallowed, but eventually we'll
            // add reporting if a --verbose flag is on.
            for result in walker {
                match result {
                    Ok(entry) => handle_entry(entry, &config),
                    _ => (),
                }
            }
        }
    }

    Ok(())
}
