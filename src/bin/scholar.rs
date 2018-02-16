#[macro_use]
extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate serde_json;

extern crate scholar;

use std::fs;

use clap::{App, Arg, ArgGroup, ArgMatches};

use scholar::request;
use scholar::scrape::{CitationDocument, ClusterDocument, SearchDocument};

mod config;
mod scrape;
mod errors;

use config::Config;
use scrape::Scrape;
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

    let cfg = Config::new(&matches);

    if matches.is_present("cluster-id") {
        let cluster_id = value_t!(matches, "cluster-id", u64).unwrap(); // validated in app()
        let query = request::ClusterQuery::new(cluster_id);
        let body = request::send_request(&query, cfg.verbose)?;
        let doc = ClusterDocument::from(&*body);
        doc.scrape(&cfg)?;

        return Ok(());
    }

    if let Some(cite_file) = matches.value_of("cite-html") {
        let file = fs::File::open(cite_file)?;
        let doc = CitationDocument::from_read(file)?;
        doc.scrape(&cfg)?;

        return Ok(());
    }

    let search_doc = if let Some(search_file) = matches.value_of("search-html") {
        let file = fs::File::open(search_file)?;
        SearchDocument::from_read(file)?
    } else {
        let mut query = request::SearchQuery::default();

        if matches.is_present("count") {
            let count = value_t!(matches, "count", u32).unwrap(); // validated in app()
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

        let body = request::send_request(&query, cfg.verbose)?;
        SearchDocument::from(&*body)
    };

    search_doc.scrape(&cfg)?;

    Ok(())
}

fn app() -> App<'static, 'static> {
    App::new(env!("CARGO_PKG_NAME"))
        .version(crate_version!())
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Maximum number of search results (default = 5)")
                .takes_value(true)
                .validator(|v| match v.parse::<u32>() {
                    Ok(v) if v > 10 => Err(String::from("The value is too large; exceeding 10")),
                    Ok(v) if v > 0 => Ok(()),
                    _ => Err(String::from("The value is not a positive integer")),
                })
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
                .help("Search only papers which contain specified words in their title (default = false)")
                .display_order(4),
        )
        .group(
            ArgGroup::with_name("search-query")
                .args(&["words", "phrase", "authors"])
                .multiple(true)
                .conflicts_with_all(&["cluster-id", "html"]),
        )
        .arg(
            Arg::with_name("cluster-id")
                .long("cluster-id")
                .help("Search a paper with this cluster ID")
                .takes_value(true)
                .validator(|v| match v.parse::<u64>() {
                    Ok(_) => Ok(()),
                    _ => Err(String::from("The value is not an integer")),
                })
                .conflicts_with("html")
                .display_order(10),
        )
        .arg(
            Arg::with_name("search-html")
                .long("search-html")
                .help(
                    "Scrape this HTML file as a search results page \
                     (possibly useful only when debugging)",
                )
                .value_name("file")
                .display_order(20),
        )
        .arg(
            Arg::with_name("cite-html")
                .long("cite-html")
                .help(
                    "Scrape this HTML file as a citers list page \
                     (possibly useful only when debugging)",
                )
                .value_name("file")
                .display_order(21),
        )
        .group(ArgGroup::with_name("html").args(&["search-html", "cite-html"]))
        .arg(
            Arg::with_name("json")
                .long("json")
                .help("Output in JSON format")
                .display_order(30),
        )
        .arg(
            Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Verbose mode")
            .display_order(22)
        )
}

fn query_exists(matches: &ArgMatches) -> bool {
    matches.is_present("search-query") || matches.is_present("html")
        || matches.is_present("cluster-id")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_arg_conflict_test() {
        use clap::ErrorKind::ArgumentConflict;

        assert_eq!(
            app()
                .get_matches_from_safe(&["prog", "--words", "foo", "--cluster-id", "0"])
                .unwrap_err()
                .kind,
            ArgumentConflict
        );

        assert_eq!(
            app()
                .get_matches_from_safe(&[
                    "prog",
                    "--phrase",
                    r#""foo bar""#,
                    "--search-html",
                    "foo.html"
                ])
                .unwrap_err()
                .kind,
            ArgumentConflict
        );

        assert_eq!(
            app()
                .get_matches_from_safe(&["prog", "--authors", "foo", "--cite-html", "foo.html"])
                .unwrap_err()
                .kind,
            ArgumentConflict
        );

        assert_eq!(
            app()
                .get_matches_from_safe(&["prog", "--cluster-id", "0", "--search-html", "foo.html"])
                .unwrap_err()
                .kind,
            ArgumentConflict
        );

        assert_eq!(
            app()
                .get_matches_from_safe(&[
                    "prog",
                    "--search-html",
                    "foo.html",
                    "--cite-html",
                    "foo.html"
                ])
                .unwrap_err()
                .kind,
            ArgumentConflict
        );
    }

    #[test]
    fn query_exists_test() {
        assert!(query_exists(&app().get_matches_from(&["prog", "--words", "foo"])));

        assert!(query_exists(&app().get_matches_from(&[
            "prog",
            "--phrase",
            r#""foo bar""#
        ])));

        assert!(query_exists(&app().get_matches_from(&[
            "prog",
            "--authors",
            "foo"
        ])));

        assert!(query_exists(&app().get_matches_from(&[
            "prog",
            "--cluster-id",
            "0"
        ])));

        assert!(query_exists(&app().get_matches_from(&[
            "prog",
            "--search-html",
            "foo.html"
        ])));

        assert!(query_exists(&app().get_matches_from(&[
            "prog",
            "--cite-html",
            "foo.html"
        ])));

        assert!(!query_exists(&app().get_matches_from(&["prog"])));

        assert!(!query_exists(&app().get_matches_from(&["prog", "--count", "1"])));
    }
}
