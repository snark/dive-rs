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

        for target in vec!["name", "path", "match", "regex"] {
            for n in matches.opt_strs(target) {
                let res = match target {
                    "name" => MatchRule::new_glob_match(&n, &GlobTarget::Name),
                    "path" => MatchRule::new_glob_match(&n, &GlobTarget::Path),
                    "match" => MatchRule::new_substr_match(&n, case_sensitive),
                    "regex" => MatchRule::new_regex_match(&n, case_sensitive),
                    _ => unreachable!(),
                };
                match res {
                    Ok(rule) => rules.push(rule),
                    _ => rule_errors.push(n),
                };
            }
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
            require_literal_separator: false,
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
