extern crate getopts;
extern crate glob;
extern crate ignore;
extern crate regex;

use std::error::Error;
use std::path::Path;

use glob::{Pattern, MatchOptions};
use ignore::{WalkBuilder, DirEntry};
use ignore::overrides::OverrideBuilder;
use regex::{Regex, RegexBuilder};

#[derive(Debug)]
pub enum GlobTarget {
    Path,
    Name,
}

#[derive(Debug)]
pub enum MatchRule {
    NameGlob(Pattern),
    PathGlob(Pattern),
    Regex(Regex),
    Substring(String),
}

impl MatchRule {
    pub fn matches(&self, path: &Path, config: &Config) -> bool {
        // Note that we are returning bool, not Option<bool>; an inability
        // to process the path in whatever fashion should be treated as
        // a failed match.
        match *self {
            MatchRule::NameGlob(ref p) => {
                let file_name = path.file_name().and_then(|f| f.to_str());
                match file_name {
                    Some(f) => p.matches_with(f, &config.glob_options),
                    None => false,
                }
            }
            MatchRule::PathGlob(ref p) => {
                let file_path = path.parent();
                match file_path {
                    Some(f) => p.matches_path_with(f, &config.glob_options),
                    None => false,
                }
            }
            MatchRule::Regex(ref re) => {
                let path_string = path.to_str();
                match path_string {
                    Some(s) => re.is_match(s),
                    None => false,
                }
            }
            MatchRule::Substring(ref substr) => {
                let path_string = path.to_str();
                match path_string {
                    Some(s) => {
                        if config.case_sensitive {
                            s.contains(substr)
                        } else {
                            let slc = s.to_lowercase();
                            slc.contains(substr)
                        }
                    }
                    None => false,
                }
            }
        }
    }
}

pub struct Config {
    pub all: bool,
    pub glob_options: MatchOptions,
    pub case_sensitive: bool,
    pub rules: Vec<MatchRule>,
}

impl Config {
    pub fn new(matches: &getopts::Matches) -> Result<Config, String> {
        let case_sensitive = matches.opt_present("case-sensitive");
        let mut rules = vec![];
        let mut errors = vec![];

        for n in matches.opt_strs("name") {
            let pattern = Pattern::new(&n);
            match pattern {
                Ok(g) => rules.push(MatchRule::NameGlob(g)),
                _ => errors.push(n),
            };
        }
        for p in matches.opt_strs("path") {
            let pattern = Pattern::new(&p);
            match pattern {
                Ok(g) => rules.push(MatchRule::PathGlob(g)),
                _ => errors.push(p),
            };
        }
        for m in matches.opt_strs("match") {
            if case_sensitive {
                rules.push(MatchRule::Substring(m));
            } else {
                rules.push(MatchRule::Substring(m.to_lowercase()));
            }
        }
        for r in matches.opt_strs("regex") {
            let pattern = RegexBuilder::new(&r)
                .case_insensitive(!case_sensitive)
                .unicode(true)
                .build();
            match pattern {
                Ok(re) => rules.push(MatchRule::Regex(re)),
                _ => errors.push(r),
            }
        }

        if errors.len() > 0 {
            return Err(errors.join(","));
        }

        let glob_options = MatchOptions {
            case_sensitive: case_sensitive,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        Ok(Config {
            all: matches.opt_present("all"),
            glob_options: glob_options,
            case_sensitive: case_sensitive,
            rules: rules,
        })
    }
}

fn handle_entry(entry: DirEntry, config: &Config) {
    // Always match if there are no path-matching rules
    let mut m = config.rules.as_slice().len() == 0;
    let p = entry.path();
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
