use std::fmt::Write;
use std::{error::Error, process};

use clap::Parser;
use colored::control;
use dictate::{
    cache::Cache,
    charset,
    cli::{Cli, Command, When},
    client,
    entry::Entry,
};
use minus::Pager;
use tokio::fs::OpenOptions;

#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|e| {
        eprintln!("dictate: {}", e);
        process::exit(1);
    });
}

async fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    configure_color(&cli.color);

    let mut pager = Pager::new();
    let mut cache = Cache::open(OpenOptions::new().read(true).write(true).create(true)).await?;

    match cli.command {
        Command::Lookup { word } => {
            let entries = fetch_entries(&mut cache, &word).await?;

            for (i, entry) in entries.iter().enumerate() {
                if i > 0 {
                    writeln!(pager)?;
                }

                writeln!(pager, "{}", entry)?;
            }
        }

        Command::Clean { cache: cache_flag } => {
            if cache_flag {
                cache.clean().await?;
            }
        }

        Command::Complete { shell } => {
            let completion = match shell.as_str() {
                "bash" => include_str!(concat!(env!("OUT_DIR"), "/bash/dictate.bash")),
                "elvish" => include_str!(concat!(env!("OUT_DIR"), "/elvish/dictate.elv")),
                "fish" => include_str!(concat!(env!("OUT_DIR"), "/fish/dictate.fish")),
                "powershell" => {
                    include_str!(concat!(env!("OUT_DIR"), "/powershell/_dictate.ps1"))
                }
                "zsh" => include_str!(concat!(env!("OUT_DIR"), "/zsh/_dictate")),
                _ => return Err(format!("shell `{}` not supported", shell).into()),
            };

            println!("{}", completion);
        }
    }

    pager.set_run_no_overflow(true)?;
    minus::page_all(pager)?;

    Ok(())
}

fn configure_color(color: &When) {
    match color {
        When::Auto => (),
        When::Never => {
            control::set_override(false);
            charset::set_override(false);
        }
        When::Always => {
            control::set_override(true);
            charset::set_override(true);
        }
    };
}

async fn fetch_entries(cache: &mut Cache, word: &str) -> Result<Vec<Entry>, Box<dyn Error>> {
    let mut entries = cache.lookup_word(word).await?;
    if entries.is_empty() {
        entries = client::lookup_word(word).await?;
        cache.append(&mut entries.clone()).await?;
    }

    Ok(entries)
}
