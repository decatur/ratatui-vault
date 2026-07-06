//! A terminal password manager: minimal and auditable.

mod commands;
mod crypt;
mod error_string;
mod git_credential_helper;
mod model;
mod prompt;
mod ssh_ask_pass;
mod ui;

use std::path::PathBuf;

use error_string::Result;

/// # Usage
/// ```text
///     ratatui-vault vault_a [vault_b]
///     ratatui-vault myvault --change
///     ratatui-vault myvault --dump | grep MyBank -A 5
///     ratatui-vault myvault --query github
/// ``````
pub fn main() -> Result<()> {
    // use ratatui_crossterm::crossterm::tty::IsTty;
    // let is_tty: bool = std::io::stdin().is_tty();
    // eprintln!("{is_tty}");

    let args = std::env::args().skip(1).collect::<Box<_>>();
    let (positional, options) = parse_args(args);
    let path = parse_vault_path();
    eprintln!("{options:?} {positional:?} {path:?}");

    if let Some(path) = &path
        && let Some(host) = ssh_ask_pass::ssh_host(&positional, &options)
    {
        ssh_ask_pass::process_command(path, host)?;
    } else if let Some(path) = &path
        && let Some(command) = git_credential_helper::command(&positional, &options)
    {
        if command == "get" {
            git_credential_helper::process_command(path)?;
        } else {
            // This is probably a "git store" command
        }
    } else if options.is_empty() {
        ui::run(positional)?;
    } else {
        assert_eq!(
            positional.len(),
            1,
            "Exactely one positional argument expected."
        );
        let path = std::path::Path::new(&positional[0]);
        assert!(path.is_file(), "Path must be a vault");

        assert_eq!(options.len(), 1, "Exactely one option argument expected.");
        let [key, value] = options.first().unwrap();
        match key.as_str() {
            "change" => {
                commands::change::run(path)?;
            }
            "dump" => {
                commands::dump::run(path)?;
            }
            "query" => {
                commands::query::run(path, value)?;
            }
            option => {
                panic!("Invalid option {option}")
            }
        };
    }
    Ok(())
}

fn split_line(line: &str) -> Option<(&str, &str)> {
    let parts: Vec<_> = line.trim().split('=').collect();
    if parts.len() == 2 {
        Some((parts[0].trim(), parts[1].trim()))
    } else {
        None
    }
}

/// Splits a flat array of strings like std::env::args() into positional and option arguments.
/// For example [foo, --foz=baz, bar] -> ([foo, bar], [[foz, baz]])
fn parse_args(args: Box<[String]>) -> (Vec<String>, Vec<[String; 2]>) {
    let mut positional = vec![];
    let mut options = vec![];
    for arg in args {
        if let Some(option) = arg.strip_prefix("--") {
            if let Some((key, value)) = option.split_once('=') {
                options.push([key.to_owned(), value.to_owned()]);
            } else {
                options.push([option.to_owned(), "".to_owned()]);
            }
        } else {
            positional.push(arg.to_owned());
        }
    }
    (positional, options)
}

fn parse_vault_path() -> Option<PathBuf> {
    match std::env::var("VAULT_PATH") {
        Ok(path) => Some(PathBuf::from(path)),
        Err(err) => match err {
            std::env::VarError::NotPresent => None,
            std::env::VarError::NotUnicode(_os_string) => {
                eprintln!("Env var VAULT_PATH does contain invalid unicode data");
                None
            }
        },
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn parse_args() {
        let line = "foo --foz=baz bar --dump";
        let args = line
            .split_ascii_whitespace()
            .map(|arg| arg.to_owned())
            .collect::<Box<_>>();
        let (positional, options) = super::parse_args(args);
        assert_eq!(options, vec![["foz", "baz"], ["dump", ""]]);
        assert_eq!(positional, vec!["foo", "bar"]);
    }
}
