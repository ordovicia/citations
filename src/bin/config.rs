use clap::ArgMatches;

#[derive(Clone)]
pub struct Config {
    pub max_result_count: Option<u32>,
    pub recursive_depth: u32,
    pub output_format: OutputFormat,
    pub verbose: bool,
}

#[derive(Clone)]
pub enum OutputFormat {
    HumanReadable,
    Json,
}

impl Config {
    pub fn new(matches: &ArgMatches) -> Self {
        use std::cmp;

        let output_format = if matches.is_present("json") {
            OutputFormat::Json
        } else {
            OutputFormat::HumanReadable
        };

        Self {
            max_result_count: value_t!(matches, "count", u32).ok(),
            recursive_depth: cmp::min(
                value_t!(matches, "recursive", u32).unwrap_or(0),
                super::MAX_RECURSIVE_DEPTH,
            ),
            output_format,
            verbose: matches.is_present("verbose"),
        }
    }
}
