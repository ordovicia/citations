#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate serde_json;

extern crate scholar;

use std::fs;

use clap::{App, Arg, ArgGroup, ArgMatches};

use scholar::request;
use scholar::scrape::{CitationDocument, IdDocument, PapersDocument, SearchDocument};

mod config;
mod errors;

use config::*;
use errors::*;

quick_main!(run);

fn run() -> Result<()> {
    let matches = app().get_matches();

    if !query_exists(&matches) {
        use clap::{Error, ErrorKind};

        app().print_help()?;
        println!("\n");
        Error::with_description("Missing query", ErrorKind::MissingRequiredArgument).exit();
    }

    let mut cfg = Config::default();

    if matches.is_present("json") {
        cfg.output_format = OutputFormat::Json;
    }

    if matches.is_present("id") {
        let id = value_t!(matches, "id", u64).unwrap_or_else(|e| e.exit());
        let query = request::IdQuery::new(id);
        let body = request::send_request(&query)?;
        let doc = IdDocument::from(&*body);

        run_id_document(&doc, &cfg)?;
    } else if let Some(cite_file) = matches.value_of("cite-html") {
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

fn app() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Maximum number of search results")
                .takes_value(true)
                .display_order(0),
        )
        .arg(
            Arg::with_name("words")
                .short("w")
                .long("words")
                .help("Search papers with these words")
                .takes_value(true)
                .display_order(1),
        )
        .arg(
            Arg::with_name("phrase")
                .short("p")
                .long("phrase")
                .help("Search papers with this exact phrase")
                .takes_value(true)
                .display_order(2),
        )
        .arg(
            Arg::with_name("authors")
                .short("a")
                .long("authors")
                .help("Search papers with these authors")
                .takes_value(true)
                .display_order(3),
        )
        .arg(
            Arg::with_name("title-only")
                .short("t")
                .long("title-only")
                .help("Search only papers which contain specified words in their title")
                .display_order(4),
        )
        .group(
            ArgGroup::with_name("search-query")
                .args(&["words", "phrase", "authors"])
                .multiple(true)
                .conflicts_with_all(&["html", "id"]),
        )
        .arg(
            Arg::with_name("search-html")
                .long("search-html")
                .help(
                    "Scrape this HTML file as search results (possibly useful only when debugging)",
                )
                .value_name("file")
                .display_order(10),
        )
        .arg(
            Arg::with_name("cite-html")
                .long("cite-html")
                .help("Scrape this HTML file as citers list (possibly useful only when debugging)")
                .value_name("file")
                .display_order(11),
        )
        .group(
            ArgGroup::with_name("html")
                .args(&["search-html", "cite-html"])
                .conflicts_with("id"),
        )
        .arg(
            Arg::with_name("id")
                .long("cluster-id")
                .help("Search a paper with this cluster ID")
                .takes_value(true)
                .display_order(20),
        )
        .arg(
            Arg::with_name("json")
                .long("json")
                .help("Output in JSON format")
                .display_order(20),
        )
}

fn query_exists(matches: &ArgMatches) -> bool {
    matches.is_present("search-query") || matches.is_present("html") || matches.is_present("id")
}

fn run_id_document(doc: &IdDocument, cfg: &Config) -> Result<()> {
    let target_paper = doc.scrape_target_paper()?;

    match cfg.output_format {
        OutputFormat::HumanReadable => {
            println!("Search result:\n");
            println!("{}\n", target_paper);
        }
        OutputFormat::Json => {
            let j = serde_json::to_string_pretty(&target_paper)?;
            println!("{}", j);
        }
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
