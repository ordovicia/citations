#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate select;

use std::env;
use std::fs::File;
use std::path::PathBuf;

use select::document::Document;

mod scrape;
mod errors;
use errors::*;

pub struct Paper {
    pub name: String,
    pub id: String,
}

quick_main!(run);

fn run() -> Result<()> {
    let html = {
        let file = get_file()?;
        Document::from_read(file)?
    };

    let target_paper = scrape::scrape_target_paper(&html)?;
    println!(
        r#""{}" (id: {}) is cited by:"#,
        target_paper.name,
        target_paper.id
    );

    Ok(())
}

fn get_file() -> Result<File> {
    let arg1 = env::args().nth(1).ok_or(ErrorKind::Cli(
        format!("Usage: {} file", env::args().nth(0).unwrap()),
    ))?;
    let path = PathBuf::from(arg1);
    let file = File::open(path)?;
    Ok(file)
}
