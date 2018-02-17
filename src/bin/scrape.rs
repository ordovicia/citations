use serde_json;

use scholar::paper::Paper;
use scholar::request::{send_request, CitationQuery};
use scholar::scrape::{CitationDocument, ClusterDocument, PapersDocument, SearchDocument};

use config::{Config, OutputFormat};
use errors::*;

macro_rules! exit_blocked {
    ($doc: ident) => {
        if $doc.is_blocked() {
            return Err(ErrorKind::Blocked.into());
        }
    }
}

pub fn scrape_cluster_doc(doc: &ClusterDocument, cfg: &Config) -> Result<()> {
    exit_blocked!(doc);

    let paper = {
        let mut p = doc.scrape_target_paper()?;

        if cfg.recursive_depth > 0 {
            p = recursive_search(&p, cfg)?;
        }

        p
    };

    match cfg.output_format {
        OutputFormat::HumanReadable => {
            println!("Result:\n");
            println!("{}\n", paper);
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&paper)?);
        }
    }

    Ok(())
}

pub fn scrape_citaiton_doc(doc: &CitationDocument, cfg: &Config) -> Result<()> {
    exit_blocked!(doc);

    let paper = {
        let mut p = doc.scrape_target_paper_with_citers()?;

        if cfg.recursive_depth > 0 {
            let new_citers = p.citers
                .unwrap()
                .iter()
                .flat_map(|c| recursive_search(c, cfg))
                .collect();
            p.citers = Some(new_citers);
        }

        p
    };

    match cfg.output_format {
        OutputFormat::HumanReadable => {
            println!("The target paper:\n");
            println!("{}\n", paper);

            println!("... is cited by:\n");
            for citer in paper.citers.unwrap() {
                println!("{}\n", citer);
            }
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&paper)?);
        }
    }

    Ok(())
}

pub fn scrape_search_doc(doc: &SearchDocument, cfg: &Config) -> Result<()> {
    exit_blocked!(doc);

    let papers = {
        let mut papers = doc.scrape_papers()?;

        if cfg.recursive_depth > 0 {
            papers = papers
                .iter()
                .flat_map(|p| recursive_search(p, cfg))
                .collect();
        }

        papers
    };

    match cfg.output_format {
        OutputFormat::HumanReadable => for paper in papers {
            println!("Result:\n");
            println!("{}\n", paper);
        },
        OutputFormat::Json => for paper in papers {
            println!("{}", serde_json::to_string_pretty(&paper)?);
        },
    }

    Ok(())
}

fn recursive_search(paper: &Paper, cfg: &Config) -> Result<Paper> {
    if cfg.recursive_depth == 0 {
        return Ok(paper.clone());
    }

    assert!(cfg.recursive_depth > 0);

    let new_cfg = {
        let mut c = cfg.clone();
        c.recursive_depth -= 1;
        c
    };

    let query = {
        let mut q = CitationQuery::new(&paper.citation_url);
        if let Some(count) = cfg.max_result_count {
            q.set_count(count);
        }
        q
    };

    let body = send_request(&query, cfg.verbose)?;
    let doc = CitationDocument::from(&*body);
    exit_blocked!(doc);

    let mut new_paper = doc.scrape_target_paper_with_citers()?;
    let new_citers = new_paper
        .citers
        .unwrap()
        .iter()
        .flat_map(|c| recursive_search(c, &new_cfg))
        .collect();
    new_paper.citers = Some(new_citers);

    Ok(new_paper)
}
