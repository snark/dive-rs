extern crate ignore;

use std::path::Path;
use ignore::Walk;

pub fn run(start_paths: Vec<std::string::String>) {
    for path_string in start_paths {
        let exists;
        {
            let path = Path::new(&path_string);
            exists = path.exists();
        }
        if !exists {
            println!("{}: No such file or directory", path_string);
        } else {
            for result in Walk::new(path_string) {
                match result {
                    Ok(entry) => println!("{}", entry.path().display()),
                    Err(err) => println!("ERROR: {}", err),
                }
            }
        }
    }
}

