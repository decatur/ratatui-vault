use crate::crypt;
use crate::error_string::{Error, Result};

enum Status {
    Initial,
    SectionFound,
}

pub fn run(path: &std::path::Path, section: &str) -> Result<()> {
    let section_not_found = || format!("Could not find section {section}");
    let password = crypt::prompt_secret("Please enter password:");
    let haystack = crypt::decrypt_from_file(path, &password)?;
    let (status, lines) = query_section(&haystack, section);

    if let Status::SectionFound = status {
        for line in lines {
            println!("{line}");
        }
        Ok(())
    } else {
        Err(Error(section_not_found()))
    }
}

fn query_section(haystack: &str, section: &str) -> (Status, Vec<String>) {
    let mut status = Status::Initial;

    let mut lines = Vec::new();
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

                lines.push(line.to_owned());
            }
        }
    }

    (Status::SectionFound, lines)
}

pub(crate) fn query_section_value(haystack: &str, section: &str, key: &str) -> Option<String> {
    let (status, lines) = query_section(haystack, section);
    // eprintln!("{lines:?}");
    if let Status::SectionFound = status {
        for line in lines {
            let parts: Vec<_> = line.trim().split('=').collect();
            if parts.len() == 2 && parts[0].trim() == key {
                return Some(parts[1].trim().to_owned());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::commands::query::{Status, query_section, query_section_value};

    #[test]
    fn test_query_section() {
        let haystack = "
[Section1]
[Section2]
    foz=baz 
    bar
[Section2]";
        let (status, lines) = query_section(haystack, "Section2");
        assert!(matches!(status, Status::SectionFound));
        assert_eq!(lines, vec!["foz=baz", "bar"]);
        // assert_eq!(positional, vec!["foo", "bar"]);
    }

    #[test]
    fn test_query_section_value() {
        let haystack = "
[Section1]
[Section2]
    foz=baz 
    bar
[Section2]";
        let value = query_section_value(haystack, "Section2", "foz");
        assert_eq!(value.unwrap(), "baz");
    }
}
