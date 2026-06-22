use std::path::PathBuf;

use crate::crypt;
use crate::error_string::{Error, Result};

pub fn run(args: Vec<String>) -> Result<()> {
    if args.len() != 2 {
        println!("The query command needs path and section arguments");
    }
    let [path, section] = [&args[0], &args[1]];
    let section_not_found = || format!("Could not find section {section}");

    let password = crypt::prompt_secret("Please enter password:");
    let haystack = crypt::decrypt_from_file(&PathBuf::from(path), &password)?;
    enum Status {
        Initial,
        SectionFound,
    }
    let mut status = Status::Initial;
    for line in haystack.lines() {
        let line = line.trim();
        match status {
            Status::Initial => {
                if line == format!("[{section}]") {
                    status = Status::SectionFound;
                }
            }
            Status::SectionFound => {
                if line.is_empty() {
                    continue;
                } else if line.starts_with("[") {
                    break;
                }

                // println!("{line}");
                let parts: Vec<_> = line.trim().split('=').collect();
                if parts.len() == 2 {
                    println!("{}={}", parts[0].trim(), parts[1].trim());
                } else {
                    println!("{line}");
                }
            }
        }
    }
    if let Status::SectionFound = status {
        Ok(())
    } else {
        Err(Error(section_not_found()))
    }
}
