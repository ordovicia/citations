use serde_json;

use config::{Config, OutputFormat};
use scholar::scrape::{CitationDocument, IdDocument, PapersDocument, SearchDocument};
use errors::*;

pub trait Scrape {
    fn scrape(&self, cfg: &Config) -> Result<()>;
}

impl Scrape for IdDocument {
    fn scrape(&self, cfg: &Config) -> Result<()> {
        let target_paper = self.scrape_target_paper()?;

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
}

impl Scrape for CitationDocument {
    fn scrape(&self, cfg: &Config) -> Result<()> {
        let target_paper = self.scrape_target_paper_with_citers()?;

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
}

impl Scrape for SearchDocument {
    fn scrape(&self, cfg: &Config) -> Result<()> {
        match cfg.output_format {
            OutputFormat::HumanReadable => {
                println!("Search result:\n");
                for paper in self.scrape_papers()? {
                    println!("{}\n", paper);
                }
            }
            OutputFormat::Json => for paper in self.scrape_papers()? {
                let j = serde_json::to_string_pretty(&paper)?;
                println!("{}", j);
            },
        }

        Ok(())
    }
}
