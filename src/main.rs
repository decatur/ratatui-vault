//! This is a binary crate for a terminal password manager which is minimal and auditable.
use std::env;

use ratatui_vault::{Result, commands, generate_password};

fn help() {
    println!("Usage:");
    println!("  ratatui-vault edit myvault");
    println!("  ratatui-vault create myvault");
    println!("  ratatui-vault change myvault");
    println!("  ratatui-vault diff vault_a vault_b");
    println!("  ratatui-vault dump myvault | grep MyBank -A 5");
    println!("  ratatui-vault query myvault https://github.com/login");
    println!("  ratatui-vault generate");
}

/// # Usage
/// ```
///     ratatui-vault edit myvault
///     ratatui-vault create myvault
///     ratatui-vault change myvault
///     ratatui-vault diff vault_a vault_b
///     ratatui-vault dump myvault | grep MyBank -A 5
///     ratatui-vault query myvault github
///     ratatui-vault generate
/// ``````
pub fn main() -> Result<()> {
    let mut args = env::args();
    if let Some(cmd) = args.nth(1) {
        match cmd.as_str() {
            "edit" => {
                ratatui_vault::ui::run(args.next())?;
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
            "generate" => {
                println!("{}", generate_password(20));
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
