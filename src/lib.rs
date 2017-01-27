pub mod config;
mod matching;

extern crate getopts;
extern crate glob;
extern crate regex;
extern crate ignore;

use ignore::{DirEntry, WalkBuilder};
use ignore::overrides::OverrideBuilder;
use std::error::Error;
use std::fs::FileType;
use std::os::unix::fs::FileTypeExt;
use std::path::Path;

fn handle_entry(entry: DirEntry, config: &config::Config) {
    // Always match if there are no path-matching rules
    let mut m = config.rules.as_slice().len() == 0;
    let p = entry.path();
    if config.filetype_filter.is_some() {
        let filetype_filter = match config.filetype_filter {
            Some(ref filter) => {
                match filter {
                    &config::Filetype::BlockSpecial => FileTypeExt::is_block_device,
                    &config::Filetype::CharacterSpecial => FileTypeExt::is_char_device,
                    &config::Filetype::Directory => FileType::is_dir,
                    &config::Filetype::FIFO => FileTypeExt::is_fifo,
                    &config::Filetype::RegularFile => FileType::is_file,
                    &config::Filetype::Socket => FileType::is_socket,
                    &config::Filetype::SymbolicLink => FileType::is_symlink,
                }
            }
            None => unreachable!(),
        };
        match entry.metadata() {
            Ok(md) => {
                if !filetype_filter(&md.file_type()) {
                    return ();
                }
            }
            Err(_) => (),
        };
    }
    for rule in config.rules.as_slice() {
        if rule.matches(p, config) {
            m = true;
            break;
        }
    }
    if m {
        println!("{}", entry.path().display());
    };
}

pub fn run(start_paths: Vec<std::string::String>,
           config: config::Config)
           -> Result<(), Box<Error>> {
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
