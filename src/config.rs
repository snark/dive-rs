extern crate getopts;

use glob::{Pattern, MatchOptions};
use matching::MatchRule;
use regex::RegexBuilder;

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
