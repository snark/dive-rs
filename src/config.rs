extern crate getopts;

use glob::MatchOptions;
use matching::{GlobTarget, MatchRule};
use std::num::ParseIntError;

#[derive(Debug)]
pub enum Filetype {
    BlockSpecial,
    CharacterSpecial,
    Directory,
    RegularFile,
    SymbolicLink,
    FIFO,
    Socket,
}

pub struct Config {
    pub all: bool,
    pub case_sensitive: bool,
    pub filetype_filter: Option<Filetype>,
    pub glob_options: MatchOptions,
    pub max_depth: Option<usize>,
    pub rules: Vec<MatchRule>,
}

impl Config {
    pub fn new(matches: &getopts::Matches) -> Result<Config, String> {

        let case_sensitive = matches.opt_present("case-sensitive");
        let filetype = try!(Config::flag_to_filetype(matches.opt_str("type")));
        let mut rules = vec![];
        let mut errors = vec![];
        let mut rule_errors = vec![];

        for target in vec!["exact", "regex", "regex-full"] {
            for n in matches.opt_strs(target) {
                let res = match target {
                    "exact" => MatchRule::new_exact_match(&n, case_sensitive),
                    "regex" => MatchRule::new_regex_match(&n, case_sensitive, false),
                    "regex-full" => MatchRule::new_regex_match(&n, case_sensitive, true),
                    _ => unreachable!(),
                };
                match res {
                    Ok(rule) => rules.push(rule),
                    _ => rule_errors.push(n),
                };
            }
        }
        for smart_match in matches.free.clone() {
            let r = if smart_match.contains("/") {
                // If we contain slashes, we need do a full-path match; further, we need
                // to wrap the final argument in asterisks if it's not a glob.
                if Config::contains_glob_characters(&smart_match) {
                    // We assume the users know what they are doing
                    MatchRule::new_glob_match(&smart_match, &GlobTarget::Path)
                } else {
                    // This is something of an edge case -- given input like
                    // "foo/bar", what do the users want? It's not clear to me
                    // at the moment, so we'll (for now) treat it as equivalent
                    // to "**/foo/*bar*".
                    let mut elements = smart_match.split('/');
                    // NB: next_back iterates R-to-L, consuming as it goes, so
                    // this strips off the final element.
                    let last = match elements.next_back() {
                        None => "".to_owned(),
                        Some("") => "*".to_owned(),
                        Some(s) => ("*".to_owned() + s) + "*",
                    };
                    let mut front = elements.filter(|piece| piece.len() != 0)
                        .map(|piece| piece.to_owned())
                        .collect::<Vec<_>>();
                    front.insert(0, "**".to_owned());
                    front.push(last);
                    let new_pattern = front.join("/");
                    MatchRule::new_glob_match(&new_pattern, &GlobTarget::Path)
                }
            } else {
                // Switch to a regex to allow us to escape characters
                if Config::contains_glob_characters(&smart_match) {
                    MatchRule::new_glob_match(&smart_match, &GlobTarget::Name)
                } else {
                    MatchRule::new_substr_match(&smart_match, case_sensitive)
                }
            };
            match r {
                Ok(rule) => rules.push(rule),
                _ => rule_errors.push(smart_match),
            };
        }
        if rule_errors.len() > 0 {
            errors = vec![format!("unable to parse matching rules: {}", rule_errors.join(", "))];
        }
        let max_depth = Config::flag_to_usize(matches.opt_str("maxdepth"));
        match max_depth {
            Err(ref e) => errors.push(format!("unable to parse maxdepth argument: {}", e)),
            _ => (),
        }

        let glob_options = MatchOptions {
            case_sensitive: case_sensitive,
            require_literal_separator: true,
            require_literal_leading_dot: false,
        };

        if errors.len() > 0 {
            return Err(errors.join("; "));
        }

        Ok(Config {
            all: matches.opt_present("all"),
            case_sensitive: case_sensitive,
            filetype_filter: filetype,
            glob_options: glob_options,
            // unwrap is safe, because errors have been caught immediately prior
            // to returning the Config struct.
            max_depth: max_depth.unwrap(),
            rules: rules,
        })
    }


    fn flag_to_usize(flag: Option<String>) -> Result<Option<usize>, ParseIntError> {
        match flag {
            None => Ok(None),
            Some(number) => Ok(Some(number.parse::<usize>()?)),
        }
    }

    fn contains_glob_characters(s: &String) -> bool {
        s.contains("*") || s.contains("[") || s.contains("]") || s.contains("?")
    }

    fn flag_to_filetype(flag: Option<String>) -> Result<Option<Filetype>, String> {
        // Ugly dereferencing per https://github.com/rust-lang/rust/issues/28606
        match flag {
            None => Ok(None),
            Some(letter) => {
                match &*letter {
                    "b" => Ok(Some(Filetype::BlockSpecial)),
                    "c" => Ok(Some(Filetype::CharacterSpecial)),
                    "d" => Ok(Some(Filetype::Directory)),
                    "f" => Ok(Some(Filetype::RegularFile)),
                    "l" => Ok(Some(Filetype::SymbolicLink)),
                    "p" => Ok(Some(Filetype::FIFO)),
                    "s" => Ok(Some(Filetype::Socket)),
                    _ => Err(format!("Unknown filetype {}", letter)),
                }
            }
        }
    }
}
