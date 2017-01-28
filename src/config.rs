extern crate getopts;

use glob::MatchOptions;
use matching::{GlobTarget, MatchRule};

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
    pub filetype_filter: Option<Filetype>,
    pub glob_options: MatchOptions,
    pub case_sensitive: bool,
    pub rules: Vec<MatchRule>,
}

impl Config {
    pub fn new(matches: &getopts::Matches) -> Result<Config, String> {

        let case_sensitive = matches.opt_present("case-sensitive");
        let filetype = try!(Config::flag_to_filetype(matches.opt_str("type")));
        let mut rules = vec![];
        let mut errors = vec![];

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
                    _ => errors.push(n),
                };
            }
        }

        if errors.len() > 0 {
            return Err(format!("Unable to parse matching rules: {}", errors.join(", ")));
        }

        let glob_options = MatchOptions {
            case_sensitive: case_sensitive,
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };

        Ok(Config {
            all: matches.opt_present("all"),
            filetype_filter: filetype,
            glob_options: glob_options,
            case_sensitive: case_sensitive,
            rules: rules,
        })
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
