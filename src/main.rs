#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate reqwest;
extern crate select;

pub mod request;
pub mod scrape;
pub mod paper;
pub mod errors;

use clap::{App, Arg, ArgGroup};

use scrape::SearchDocument;
use errors::*;

quick_main!(run);

fn run() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Maximum number of results")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("words")
                .short("w")
                .long("words")
                .help("Search papers with these words")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("phrase")
                .short("p")
                .long("phrase")
                .help("Search papers with this exact phrase")
                .takes_value(true),
        )
        .group(
            ArgGroup::with_name("words_phrase")
                .args(&["words", "phrase"])
                .multiple(true)
                .required(true),
        )
        .after_help("Either words or phrase is required")
        .get_matches();

    let mut query = request::SearchQuery::new();

    if let Some(_) = matches.value_of("count") {
        let count = value_t!(matches, "count", u32).unwrap_or_else(|e| e.exit());
        query.set_count(count);
    }
    if let Some(words) = matches.value_of("words") {
        query.set_words(words.to_string());
    }
    if let Some(phrase) = matches.value_of("phrase") {
        query.set_phrase(phrase.to_string());
    }

    let body = request::send_query(&query)?;
    let search_doc = SearchDocument::from(&body as &str);
    for paper in search_doc.scrape_papers()? {
        println!(r#""{}" (id: {})"#, paper.title, paper.id);
    }

    Ok(())
}
