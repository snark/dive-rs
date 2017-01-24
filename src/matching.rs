extern crate glob;
extern crate regex;

use config;
use glob::Pattern;
use regex::Regex;
use std::path::Path;

#[derive(Debug)]
pub enum MatchRule {
    NameGlob(Pattern),
    PathGlob(Pattern),
    Regex(Regex),
    Substring(String),
}

impl MatchRule {
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
