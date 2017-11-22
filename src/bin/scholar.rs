#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate serde_json;

extern crate scholar;

use std::fs;

use clap::{App, Arg, ArgGroup};

use scholar::request;
use scholar::scrape::{CitationDocument, SearchDocument};

mod config;
mod errors;

use config::*;
use errors::*;

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
        .arg(
            Arg::with_name("title-only")
            .short("t")
            .long("title-only")
            .help("Search only papers which contain specified words in their title")
            .display_order(4)
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
        .arg(
            Arg::with_name("json")
            .long("json")
            .help("Output in JSON format")
            .display_order(20)
            )
        .get_matches();

    let mut cfg = Config::default();

    if matches.is_present("json") {
        cfg.output_format = OutputFormat::Json;
    }

    if let Some(cite_file) = matches.value_of("cite-html") {
        let file = fs::File::open(cite_file)?;
        let doc = CitationDocument::from_read(file)?;

        run_citation_document(&doc, &cfg)?;
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
            if matches.is_present("title-only") {
                query.set_title_only(true);
            }

            let body = request::send_request(&query)?;
            SearchDocument::from(&*body)
        };

        run_search_document(&search_doc, &cfg)?;
    }

    Ok(())
}

fn run_citation_document(doc: &CitationDocument, cfg: &Config) -> Result<()> {
    let target_paper = doc.scrape_target_paper_with_citers()?;

    match cfg.output_format {
        OutputFormat::HumanReadable => {
            println!("The target paper:\n");
            println!("{}\n", target_paper);

            println!("... is cited by:\n");
            for citer in target_paper.citers.unwrap() {
                println!("{}\n", citer);
            }
        }
        OutputFormat::Json => {
            let j = serde_json::to_string_pretty(&target_paper)?;
            println!("{}", j);
        }
    }

    Ok(())
}

fn run_search_document(doc: &SearchDocument, cfg: &Config) -> Result<()> {
    match cfg.output_format {
        OutputFormat::HumanReadable => {
            println!("Search result:\n");
            for paper in doc.scrape_papers()? {
                println!("{}\n", paper);
            }
        }
        OutputFormat::Json => for paper in doc.scrape_papers()? {
            let j = serde_json::to_string_pretty(&paper)?;
            println!("{}", j);
        },
    }

    Ok(())
}
