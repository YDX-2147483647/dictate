use std::{error::Error, process};

use atty::Stream;
use clap::Parser;
use colored::control;
use dictate::{
    cache::Cache,
    cli::{Args, When},
    client,
    entry::Charset,
};
use tokio::fs::OpenOptions;

#[tokio::main]
async fn main() {
    run().await.unwrap_or_else(|e| {
        eprintln!("dictate: {}", e.to_string());
        process::exit(1);
    });
}

async fn run() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let charset = match args.color {
        When::Auto => {
            if atty::is(Stream::Stdout) {
                Charset {
                    list: "•".to_string(),
                    section_left: "".to_string(),
                    section_right: "".to_string(),
                }
            } else {
                Charset {
                    list: "*".to_string(),
                    section_left: "<".to_string(),
                    section_right: ">".to_string(),
                }
            }
        }
        When::Never => {
            control::set_override(false);

            Charset {
                list: "*".to_string(),
                section_left: "<".to_string(),
                section_right: ">".to_string(),
            }
        }
        When::Always => {
            control::set_override(true);

            Charset {
                list: "•".to_string(),
                section_left: "".to_string(),
                section_right: "".to_string(),
            }
        }
    };

    let mut cache = Cache::open(OpenOptions::new().read(true).write(true).create(true)).await?;
    let mut entries = cache.lookup_word(&args.word).await?;
    if entries.is_empty() {
        entries = client::lookup_word(&args.word).await?;
        let entries = entries.clone();
        cache.append(&mut entries.clone()).await?;
    }

    for mut entry in entries.into_iter() {
        println!();
        println!("{}", entry.charset(charset.clone()));
    }

    Ok(())
}
