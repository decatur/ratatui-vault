/// Usage:
///     cargo run
///
use std::env;

mod commands;
mod crypt;
mod error_string;
mod model;
mod ui;

use error_string::Result;

fn help() {
    println!("Usage:");
    println!("  edit myvault");
    println!("  create myvault");
    println!("  change myvault");
    println!("  diff vault_a vault_b");
    println!("  dump myvault | grep MyBank -A 5");
    println!("  query myvault https://github.com/login");
    println!("  generate");
}

fn main() -> Result<()> {
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
            "generate" => {
                println!("{}", crypt::generate_password(20));
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
