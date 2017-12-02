//! Scrape HTML document to get information of papers.

use std::io;
use std::ops::Deref;

use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Class, Name, Predicate, Text};

use paper::Paper;
use errors::*;

pub trait PapersDocument {
    fn scrape_papers(&self) -> Result<Vec<Paper>>;
}

impl PapersDocument for Document {
    fn scrape_papers(&self) -> Result<Vec<Paper>> {
        // <div id="gs_res_ccl_mid">
        //   <div class="gs_ri">
        //     paper
        //   </div>
        //   ...
        // </div>

        let pos = Attr("id", "gs_res_ccl_mid").descendant(Class("gs_ri"));
        let nodes = self.find(pos);

        let mut papers = Vec::with_capacity(10);
        for n in nodes {
            papers.push(scrape_paper_one(&n)?);
        }

        Ok(papers)
    }
}

macro_rules! impl_from_to_document {
    ($struct: ident) => {
        impl Deref for $struct {
            type Target = Document;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<'a> From<&'a str> for $struct {
            fn from(s: &str) -> Self {
                let document = Document::from(s);
                $struct(document)
            }
        }

        impl $struct {
            pub fn new(document: Document) -> Self {
                $struct(document)
            }

            // like Document::from_read()
            pub fn from_read<R: io::Read>(readable: R) -> Result<Self> {
                let document = Document::from_read(readable)?;
                Ok(Self::new(document))
            }
        }
    }
}

macro_rules! try_html {
    ($a: expr) => { $a.ok_or(ErrorKind::BadHtml)? }
}

pub struct SearchDocument(Document);
impl_from_to_document!(SearchDocument);

pub struct CitationDocument(Document);
impl_from_to_document!(CitationDocument);

impl CitationDocument {
    pub fn scrape_target_paper(&self) -> Result<Paper> {
        let node = {
            let pos = Attr("id", "gs_rt_hdr")
                .child(Name("h2"))
                .child(Name("a").or(Text));
            try_html!(self.find(pos).nth(0))
        };

        let title = node.text();
        let id = {
            let id_url = try_html!(node.attr("href"));
            parse_id_from_url(id_url)?
        };

        Ok(Paper::new(&title, id))
    }

    pub fn scrape_target_paper_with_citers(&self) -> Result<Paper> {
        let target_paper = self.scrape_target_paper()?;
        let citers = self.scrape_papers()?;

        let mut paper = target_paper;
        paper.citers = Some(citers);
        Ok(paper)
    }
}

pub struct IdDocument(Document);
impl_from_to_document!(IdDocument);

impl IdDocument {
    pub fn scrape_target_paper(&self) -> Result<Paper> {
        let pos = Attr("id", "gs_res_ccl_mid").descendant(Class("gs_ri"));
        let node = self.find(pos).next().unwrap();
        scrape_paper_one(&node)
    }
}

fn scrape_paper_one(node: &Node) -> Result<Paper> {
    let (title, link) = scrape_title_and_link(node);
    let (id, c) = scrape_id_and_citation(node)?;

    let mut paper = Paper::new(&title, id);
    paper.link = link;
    paper.citation_count = Some(c);

    Ok(paper)
}

fn scrape_title_and_link(node: &Node) -> (String, Option<String>) {
    // There are (at least) two formats.
    //
    // 1. Link to a paper or something:
    //
    // <h3 class="gs_rt">
    //   <span>
    //       something
    //   </span>
    //   <a href="http://paper.pdf">
    //     Title of paper or something
    //   </a>
    // </h3>
    //
    // 'span' may not exists.
    //
    // 2. Not a link:
    //
    // <h3 class="gs_rt">
    //   <span>
    //       something
    //   </span>
    //   Title of paper or something
    // </h3>

    let pos = Class("gs_rt").child(Name("a"));
    if let Some(n) = node.find(pos).nth(0) {
        // 1. Link to a paper or something
        let title = n.text();
        let link = n.attr("href");
        (title, link.map(ToOwned::to_owned))
    } else {
        // 2. Not a link
        let children = node.find(Class("gs_rt")).into_selection().children();
        let text_nodes = children.filter(|n: &Node| {
            if let Some(name) = n.name() {
                name != "span"
            } else {
                true
            }
        });
        let concated_text = text_nodes
            .into_iter()
            .map(|n| n.text())
            .collect::<String>()
            .trim()
            .to_string();
        (concated_text, None)
    }
}

// Scrape article footer for
//
// * cluster id, and
// * citation count
fn scrape_id_and_citation(node: &Node) -> Result<(u64, u32)> {
    // Footer format:
    //
    // <div class="gs_fl">
    //   (something)
    //   <a href="/scholar?cites=000000>Cited by 999</a>
    //   (something)
    // </div>

    let pos = Class("gs_fl");
    let footers = try_html!(node.find(pos).nth(0)).children();

    let citation_node = footers
        .into_selection()
        .filter(|n: &Node| {
            if let Some(id_url) = n.attr("href") {
                parse_id_from_url(id_url).is_ok()
            } else {
                false
            }
        })
        .first();
    let citation_node = try_html!(citation_node);

    let id = {
        let id_url = citation_node.attr("href").unwrap();
        parse_id_from_url(id_url).unwrap()
    };

    let citation_count = parse_citation_count(&citation_node.text())?;

    Ok((id, citation_count))
}

fn parse_id_from_url(url: &str) -> Result<u64> {
    use regex::Regex;

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(cluster|cites)=(\d+)").unwrap();
    }

    let caps = try_html!(RE.captures(url));
    let id = {
        let id = try_html!(caps.get(2));
        id.as_str().parse()?
    };

    Ok(id)
}

fn parse_citation_count(text: &str) -> Result<u32> {
    use regex::Regex;

    lazy_static! {
        static ref RE: Regex = Regex::new(r"[^\d]+(\d+)").unwrap();
    }

    let caps = try_html!(RE.captures(text));
    let count = {
        let count = try_html!(caps.get(1));
        count.as_str().parse().unwrap()
    };

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_id_from_url_pass() {
        assert_eq!(parse_id_from_url("cluster=123456").unwrap(), 123456);
        assert_eq!(parse_id_from_url("scholar?cluster=654321").unwrap(), 654321);
        assert_eq!(
            parse_id_from_url("scholar?cluster=222222&foo=bar").unwrap(),
            222222
        );
    }

    #[test]
    fn parse_id_from_url_fail() {
        assert!(parse_id_from_url("foo").is_err());
        assert!(parse_id_from_url("claster=000000").is_err());
        assert!(parse_id_from_url("cluster=aaaaaa").is_err());
    }

    #[test]
    fn parse_citation_count_pass() {
        assert_eq!(parse_citation_count("Cited by 111").unwrap(), 111);
        assert_eq!(parse_citation_count("引用元 222").unwrap(), 222);
    }

    #[test]
    fn parse_citation_count_fail() {
        assert!(parse_citation_count("foo").is_err());
    }

    #[test]
    fn search_document_scrape_test() {
        use std::fs;

        let papers = {
            let file = fs::File::open("src/test_html/quantum_theory.html").unwrap();
            let doc = SearchDocument::from_read(file).unwrap();
            doc.scrape_papers().unwrap()
        };

        assert_eq!(papers.len(), 10);

        assert_eq!(papers[0], {
            let mut paper = Paper::new(
                "Quantum field theory and critical phenomena",
                16499695044466828447,
            );
            paper.link = Some(String::from("http://cds.cern.ch/record/2280881"));
            paper.citation_count = Some(4821);
            paper
        });

        assert_eq!(papers[1], {
            let mut paper = Paper::new("Quantum theory of solids", 8552492368061991976);
            paper.citation_count = Some(4190);
            paper
        });

        assert_eq!(papers[2], {
            let mut paper = Paper::new(
                "Significance of electromagnetic potentials in the quantum theory",
                5545735591029960915,
            );
            paper.link = Some(String::from(
                "https://journals.aps.org/pr/abstract/10.1103/PhysRev.115.485",
            ));
            paper.citation_count = Some(6961);
            paper
        });
    }

    #[test]
    fn citation_document_scrape_test() {
        use std::fs;

        let doc = {
            let file = fs::File::open("src/test_html/quantum_theory_citations.html").unwrap();
            CitationDocument::from_read(file).unwrap()
        };

        let target_paper = doc.scrape_target_paper().unwrap();
        let citer_papers = doc.scrape_papers().unwrap();

        assert_eq!(
            target_paper,
            Paper::new(
                "Significance of electromagnetic potentials in the quantum theory",
                5545735591029960915,
            )
        );

        assert_eq!(citer_papers.len(), 10);

        assert_eq!(citer_papers[0], {
            let mut paper = Paper::new(
                "Quantal phase factors accompanying adiabatic changes",
                15570691018430890829,
            );
            paper.link = Some(String::from(
                "http://rspa.royalsocietypublishing.org/content/royprsa/392/1802/45.full.pdf",
            ));
            paper.citation_count = Some(7813);
            paper
        });

        assert_eq!(citer_papers[1], {
            let mut paper = Paper::new(
                "Multiferroics: a magnetic twist for ferroelectricity",
                9328505180409005573,
            );
            paper.link = Some(String::from(
                "https://www.nature.com/nmat/journal/v6/n1/abs/nmat1804.html",
            ));
            paper.citation_count = Some(3232);
            paper
        });

        assert_eq!(citer_papers[2], {
            let mut paper = Paper::new("Quantum field theory", 14398189842493937255);
            paper.link = Some(String::from(
                "https://books.google.co.jp/books?\
                 hl=en&lr=&id=nnuW_kVJ500C&oi=fnd&pg=PR17\
                 &ots=vrupeDXT-V&sig=MofOsrk4Hh9qXjkS_WuQ7jHr2sY",
            ));
            paper.citation_count = Some(2911);
            paper
        });
    }
}
