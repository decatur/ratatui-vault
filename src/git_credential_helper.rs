// See [Credential Storage](https://git-scm.com/book/en/v2/Git-Tools-Credential-Storage.html#_credential_caching)

use std::{io::BufRead, path::Path};

use crate::{Result, crypt::SecretString, error_string::Error};

pub(super) fn command(positional: &[String], options: &[[String; 2]]) -> Option<String> {
    if positional.len() == 1 && options.is_empty() {
        let command = &positional[0];
        if command == "get" || command == "store" {
            Some(command.to_owned())
        } else {
            None
        }
    } else {
        None
    }
}

pub struct GitQuery {
    pub username: String,
    pub protocol: String,
    pub host: String,
}

impl GitQuery {
    fn section_header(&self) -> String {
        let GitQuery {
            username,
            protocol,
            host,
        } = self;
        format!("{protocol}://{username}@{host}")
    }
}

pub(super) fn process_command(path: &Path) -> Result<()> {
    let (query, password) = git_credential(path)?;
    let GitQuery {
        username,
        protocol,
        host,
    } = query;
    println!("protocol={protocol}");
    println!("host={host}");
    println!("username={username}");
    println!("password={}", password.plaintext());

    Ok(())
}

pub(super) fn git_credential(path: &Path) -> Result<(GitQuery, SecretString)> {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let git_query = parse_git_query(&mut handle)?;
    let section = git_query.section_header();
    let master_password = super::prompt::show(&section)?;
    let haystack = super::crypt::decrypt_from_file(path, &master_password)?;
    let credential = super::commands::query::query_section_value(&haystack, &section, "password");
    credential
        .map(|c| (git_query, SecretString::new(c)))
        .ok_or(Error(format!(
            "Could not find git password in section {section}"
        )))
}

/// git sends these lines, for example
///      protocol=https
///      host=myhost.com
///      username=myuser
///      wwwauth[]=Basic realm="GitHub"
/// [parse_git_query] will then return https://myuser@myhost.com
fn parse_git_query<T>(input: &mut T) -> super::Result<GitQuery>
where
    T: BufRead,
{
    let mut protocol = None;
    let mut username = None;
    let mut host = None;
    let mut line = String::new();
    loop {
        input.read_line(&mut line)?;
        if line.trim().is_empty() {
            break;
        }
        match super::split_line(&line) {
            Some(("protocol", value)) => protocol = Some(value.to_owned()),
            Some(("username", value)) => username = Some(value.to_owned()),
            Some(("host", value)) => host = Some(value.to_owned()),
            _ => (),
        }
        line.clear();
    }

    if let (Some(protocol), Some(username), Some(host)) = (protocol, username, host) {
        Ok(GitQuery {
            username,
            protocol,
            host,
        })
    } else {
        Err(Error("Could not parse git query".to_owned()))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_git_query() {
        let mut stdin = std::io::BufReader::new(
            "protocol=https
            host=myhost.com
            username=myuser
            wwwauth[]=Basic realm=\"GitHub\""
                .as_bytes(),
        );

        let url = super::parse_git_query(&mut stdin);
        assert_eq!(url.unwrap().section_header(), "https://myuser@myhost.com");
    }
}
