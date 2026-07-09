use std::path::Path;

use crate::{Result, crypt::SecretString, error_string::Error};

pub(super) fn ssh_host(positional: &[String], options: &[[String; 2]]) -> Option<String> {
    if positional.len() == 1 && options.is_empty() {
        let arg = positional[0].as_str();

        if arg.starts_with("(")
            && let Some(index) = arg.find(')')
        {
            Some(arg[1..index].to_owned())
        } else {
            None
        }
    } else {
        None
    }
}

pub(super) fn process_command(path: &Path, host: String) -> Result<()> {
    let section = &host;
    let master_password = super::prompt::show(section)?;
    let haystack = super::crypt::decrypt_from_file(path, &master_password)?;
    let credential = super::commands::query::query_section_value(&haystack, section, "password");
    let s = credential.map(SecretString::new).ok_or(Error(format!(
        "Could not find ssh_password in section {section}"
    )))?;
    // eprintln!("{}", s.plaintext());
    println!("{}", s.plaintext());
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_ssh_host() {
        let host = super::ssh_host(&["(user@a.b.de) Password: ".to_owned()], &[]);
        assert_eq!(host.unwrap(), "user@a.b.de");

        let host = super::ssh_host(&["(user@a.b.de)".to_owned()], &[]);
        assert_eq!(host.unwrap(), "user@a.b.de");
    }
}
