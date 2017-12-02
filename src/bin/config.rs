use clap::ArgMatches;

pub struct Config {
    pub output_format: OutputFormat,
}

pub enum OutputFormat {
    HumanReadable,
    Json,
}

impl Config {
    pub fn new(matches: &ArgMatches) -> Self {
        let output_format = if matches.is_present("json") {
            OutputFormat::Json
        } else {
            OutputFormat::HumanReadable
        };

        Self { output_format }
    }
}
