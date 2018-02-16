use serde_json;

use config::{Config, OutputFormat};

use scholar;
use scholar::paper::Paper;
use scholar::request::CitationQuery;
use scholar::scrape::{CitationDocument, ClusterDocument, PapersDocument, SearchDocument};
use errors::*;

pub trait Scrape {
    fn scrape(&self, cfg: &Config) -> Result<()>;
}

impl Scrape for ClusterDocument {
    fn scrape(&self, cfg: &Config) -> Result<()> {
        let target_paper = self.scrape_target_paper()?;

        match cfg.output_format {
            OutputFormat::HumanReadable => {
                println!("Search result:\n");
                println!("{}\n", target_paper);
            }
            OutputFormat::Json => {
                // TODO
                println!("{}", serde_json::to_string_pretty(&target_paper)?);
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

                    if cfg.recursive_depth > 0 {
                        recursive_search(&citer, cfg)?;
                    }
                }
            }
            OutputFormat::Json => {
                // TODO
                println!("{}", serde_json::to_string_pretty(&target_paper)?);

                // TODO
            }
        }

        Ok(())
    }
}

impl Scrape for SearchDocument {
    fn scrape(&self, cfg: &Config) -> Result<()> {
        let papers = self.scrape_papers()?;

        match cfg.output_format {
            OutputFormat::HumanReadable => {
                println!("Search result:\n");
                for paper in papers {
                    println!("{}\n", paper);

                    if cfg.recursive_depth > 0 {
                        recursive_search(&paper, cfg)?;
                    }
                }
            }
            OutputFormat::Json => for paper in papers {
                println!("{}", serde_json::to_string_pretty(&paper)?);

                if cfg.recursive_depth > 0 {
                    recursive_search(&paper, cfg)?;
                }
            },
        }

        Ok(())
    }
}

fn recursive_search(paper: &Paper, cfg: &Config) -> Result<()> {
    let mut query = CitationQuery::new(&paper.citation_url);
    if let Some(count) = cfg.max_result_count {
        query.set_count(count);
    }

    let body = scholar::request::send_request(&query, cfg.verbose)?;
    let doc = CitationDocument::from(&*body);

    let mut new_cfg = cfg.clone();
    new_cfg.recursive_depth -= 1;
    doc.scrape(&new_cfg)
}
