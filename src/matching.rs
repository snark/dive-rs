extern crate glob;
extern crate regex;

use config;
use glob::Pattern;
use regex::{Regex, RegexBuilder};
use std::path::Path;

#[derive(Debug)]
pub enum MatchRule {
    NameGlob(Pattern),
    PathGlob(Pattern),
    Regex(Regex),
    Substring(String),
}

#[derive(Debug)]
pub enum GlobTarget {
    Name,
    Path,
}

impl MatchRule {
    pub fn new_glob_match(arg: &String, target: &GlobTarget) -> Result<MatchRule, String> {
        let pattern = Pattern::new(arg);
        match pattern {
            Ok(g) => {
                match target {
                    &GlobTarget::Name => Ok(MatchRule::NameGlob(g)),
                    &GlobTarget::Path => Ok(MatchRule::PathGlob(g)),
                }
            }
            Err(_) => Err(format!("Unable to parse pattern {}", arg)),
        }
    }

    pub fn new_regex_match(arg: &String, case_sensitive: bool) -> Result<MatchRule, String> {
        let pattern = RegexBuilder::new(arg)
            .case_insensitive(!case_sensitive)
            .unicode(true)
            .build();
        match pattern {
            Ok(re) => Ok(MatchRule::Regex(re)),
            Err(_) => Err(format!("Unable to parse regex {}", arg)),
        }
    }

    pub fn new_substr_match(arg: &String, case_sensitive: bool) -> Result<MatchRule, String> {
        if case_sensitive {
            Ok(MatchRule::Substring(arg.clone()))
        } else {
            Ok(MatchRule::Substring(arg.clone().to_lowercase()))
        }
    }

    pub fn matches(&self, path: &Path, config: &config::Config) -> bool {
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
