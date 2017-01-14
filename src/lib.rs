extern crate getopts;
extern crate ignore;

use std::error::Error;
use std::path::Path;
use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;

pub struct Config {
    pub all: bool,
}

impl Config {
    pub fn new(matches: &getopts::Matches) -> Config {
        Config { all: matches.opt_present("a") }
    }
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
            for result in walker {
                match result {
                    Ok(entry) => println!("{}", entry.path().display()),
                    Err(err) => println!("ERROR: {}", err),
                }
            }
        }
    }

    Ok(())
}
