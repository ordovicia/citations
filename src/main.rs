#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;

extern crate scholar;

use std::fs;

use clap::{App, Arg, ArgGroup};

use scholar::errors::*;
use scholar::request;
use scholar::scrape::{CitersDocument, SearchDocument};

quick_main!(run);

fn run() -> Result<()> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Maximum number of results")
                .takes_value(true)
                .display_order(0)
        )
        .arg(
            Arg::with_name("words")
                .short("w")
                .long("words")
                .help("Search papers with these words")
                .takes_value(true)
                .display_order(1)
        )
        .arg(
            Arg::with_name("phrase")
                .short("p")
                .long("phrase")
                .help("Search papers with this exact phrase")
                .takes_value(true)
                .display_order(2)
        )
        .arg(
            Arg::with_name("authors")
            .short("a")
            .long("authors")
            .help("Search papers with these authors")
            .takes_value(true)
            .display_order(3)
            )
        .group(
            ArgGroup::with_name("search-query")
                .args(&["words", "phrase", "authors"])
                .multiple(true),
        )
        .arg(
            Arg::with_name("search-html")
                .long("search-html")
                .help("HTML file of search results")
                .takes_value(true)
                .display_order(10)
        )
        .arg(
            Arg::with_name("cite-html")
                .long("cite-html")
                .help("HTML file of citers list")
                .takes_value(true)
                .display_order(11)
        )
        .group(ArgGroup::with_name("html").args(&["search-html", "cite-html"]))
        // .group(
        //     ArgGroup::with_name("input")
        //         .args(&["search-query", "html"])
        //         .required(true),
        // )
        .get_matches();

    if let Some(citers_file) = matches.value_of("cite-html") {
        let file = fs::File::open(citers_file)?;
        let doc = CitersDocument::from_read(file)?;

        run_citers_document(&doc)?;
    } else {
        let search_doc = if let Some(search_file) = matches.value_of("search-html") {
            let file = fs::File::open(search_file)?;
            SearchDocument::from_read(file)?
        } else {
            let mut query = request::SearchQuery::default();

            if matches.is_present("count") {
                let count = value_t!(matches, "count", u32).unwrap_or_else(|e| e.exit());
                query.set_count(count);
            }
            if let Some(words) = matches.value_of("words") {
                query.set_words(words);
            }
            if let Some(phrase) = matches.value_of("phrase") {
                query.set_phrase(phrase);
            }
            if let Some(authors) = matches.value_of("authors") {
                query.set_authors(authors);
            }

            let body = request::send_request(&query)?;
            SearchDocument::from(&body as &str)
        };

        run_search_document(&search_doc)?;
    }

    Ok(())
}

fn run_citers_document(doc: &CitersDocument) -> Result<()> {
    println!("The target paper:\n");
    let target_paper = doc.scrape_target_paper()?;
    println!("{}\n", target_paper);

    println!("... is cited by:\n");
    for paper in doc.scrape_papers()? {
        println!("{}\n", paper);
    }

    Ok(())
}

fn run_search_document(doc: &SearchDocument) -> Result<()> {
    for paper in doc.scrape_papers()? {
        println!("{}\n", paper);
    }

    Ok(())
}
