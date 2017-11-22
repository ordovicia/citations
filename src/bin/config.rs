pub enum OutputFormat {
    HumanReadable,
    Json,
}

pub struct Config {
    pub output_format: OutputFormat,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::HumanReadable,
        }
    }
}
