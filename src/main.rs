//! A terminal password manager: minimal and auditable.
use std::env;

mod commands;
mod crypt;
mod error_string;
mod model;
mod ui;

use error_string::Result;

fn help() {
    println!("Usage:");
    println!("  ratatui-vault edit myvault");
    println!("  ratatui-vault create myvault");
    println!("  ratatui-vault change myvault");
    println!("  ratatui-vault diff vault_a vault_b");
    println!("  ratatui-vault dump myvault | grep MyBank -A 5");
    println!("  ratatui-vault query myvault https://github.com/login");
}

/// # Usage
/// ```text
///     ratatui-vault edit myvault
///     ratatui-vault create myvault
///     ratatui-vault change myvault
///     ratatui-vault diff vault_a vault_b
///     ratatui-vault dump myvault | grep MyBank -A 5
///     ratatui-vault query myvault github
/// ``````
pub fn main() -> Result<()> {
    let mut args = env::args();
    if let Some(cmd) = args.nth(1) {
        match cmd.as_str() {
            "edit" => {
                crate::ui::run(args.next())?;
            }
            "create" => {
                commands::create::run(args.next())?;
            }
            "change" => {
                commands::change::run(args.next())?;
            }
            "dump" => {
                commands::dump::run(args.next())?;
            }
            "diff" => {
                commands::diff::run(args.take(2).collect::<Vec<_>>());
            }
            "query" => {
                commands::query::run(args.take(2).collect::<Vec<_>>())?;
            }
            cmd => {
                println!("Invalid command {cmd}");
                help();
            }
        }
    } else {
        help();
    }
    Ok(())
}
