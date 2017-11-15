#![allow(dead_code)]
#![allow(unused_imports)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate reqwest;
extern crate select;

use std::env;
use std::fs::File;
use std::path::PathBuf;

use select::document::Document;

mod request;
mod scrape;
mod errors;

use scrape::CitingPaperDocument;
use errors::*;

pub struct Paper {
    pub name: String,
    pub id: String,
}

quick_main!(run);

fn run() -> Result<()> {
    // let mut query = request::SearchQuery::new();
    // query.set_count(2);
    // query.set_words(String::from("quantum pohe"));
    // let body = request::send_query(&query)?;
    // println!("{}", body);

    let citings = {
        let file = get_file()?;
        let doc = Document::from_read(file)?;
        CitingPaperDocument(doc)
    };

    let target_paper = citings.scrape_target_paper()?;
    println!(
        r#""{}" (id: {}) is cited by:"#,
        target_paper.name,
        target_paper.id
    );

    for paper in citings.scrape_cite_papers()? {
        println!(r#""{}" (id: {})"#, paper.name, paper.id);
    }

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
