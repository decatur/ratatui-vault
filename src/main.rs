//! A terminal password manager: minimal and auditable.
use std::{collections::HashMap, env};

mod commands;
mod crypt;
mod error_string;
mod model;
mod ui;

use error_string::Result;

// fn help() {
//     println!("Usage:");
//     println!("  ratatui-vault edit vault_a [vault_b]");
//     println!("  ratatui-vault change myvault");
//     println!("  ratatui-vault dump myvault | grep MyBank -A 5");
//     println!("  ratatui-vault query myvault https://github.com/login");
// }

/// # Usage
/// ```text
///     ratatui-vault edit vault_a [vault_b]
///     ratatui-vault change myvault
///     ratatui-vault dump myvault | grep MyBank -A 5
///     ratatui-vault query myvault github
/// ``````
pub fn main() -> Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let (positional, options) = parse_args(args);

    println!("{options:?} {positional:?}");

    if options.is_empty() {
        crate::ui::run(positional)?;
    } else {
        assert_eq!(
            positional.len(),
            1,
            "Exactely one positional argument expected."
        );
        let path = &positional[0];
        assert_eq!(options.len(), 1, "Exactely one option argument expected.");
        let (key, value) = options.into_iter().next().unwrap();
        match key.as_str() {
            "change" => {
                commands::change::run(&path)?;
            }
            "dump" => {
                commands::dump::run(&path)?;
            }
            "query" => {
                commands::query::run(&path, &value)?;
            }
            option => {
                panic!("Invalid option {option}")
            }
        };
    }
    Ok(())
}

fn parse_args(args: Vec<String>) -> (Vec<String>, HashMap<String, String>) {
    let mut positional = vec![];
    let mut options = HashMap::new();
    for arg in args {
        if arg.starts_with("--") {
            let parts = arg[2..].split('=').collect::<Vec<_>>();
            assert!(!parts.is_empty() && parts.len() <= 2);
            let key = parts[0];
            let value = if parts.len() == 1 { key } else { parts[1] };
            options.insert(key.to_owned(), value.to_owned());
        } else {
            positional.push(arg.to_owned());
        }
    }
    (positional, options)
}

#[cfg(test)]
mod tests {

    #[test]
    fn parse_args() {
        let line = "foo --foo=baz bar";
        let args = line
            .split_ascii_whitespace()
            .map(|arg| arg.to_owned())
            .collect::<Vec<_>>();
        let (positional, options) = super::parse_args(args);
        assert_eq!(options.get("foo").unwrap().as_str(), "baz");
        assert_eq!(positional, vec!["foo", "bar"]);
    }
}
