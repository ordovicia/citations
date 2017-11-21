use std::io;
use std::ops::Deref;

use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Class, Name, Predicate, Text};

use paper::Paper;
use errors::*;

macro_rules! try_html {
    ($a: expr) => { $a.ok_or(ErrorKind::BadHtml)? }
}

pub struct SearchDocument(Document);
pub struct CitationDocument(SearchDocument);

impl Deref for SearchDocument {
    type Target = Document;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<&'a str> for SearchDocument {
    fn from(s: &str) -> Self {
        let document = Document::from(s);
        SearchDocument(document)
    }
}

impl SearchDocument {
    pub fn new(document: Document) -> Self {
        SearchDocument(document)
    }

    pub fn from_read<R: io::Read>(readable: R) -> Result<Self> {
        let document = Document::from_read(readable)?;
        Ok(Self::new(document))
    }

    pub fn scrape_papers(&self) -> Result<Vec<Paper>> {
        // ```ignore
        // <div id="gs_res_ccl_mid">
        //   <div class="gs_ri">
        //     paper
        //   </div>
        //   ...
        // </div>
        // ```

        let pos = Attr("id", "gs_res_ccl_mid").descendant(Class("gs_ri"));
        let nodes = self.find(pos);

        let mut papers = Vec::with_capacity(10);
        for n in nodes {
            papers.push(Self::scrape_paper_one(&n)?);
        }

        Ok(papers)
    }

    fn scrape_paper_one(node: &Node) -> Result<Paper> {
        let title = Self::scrape_title(node);
        let (id, c) = Self::scrape_id_and_citation(node)?;

        Ok(Paper {
            title,
            id,
            citation_count: Some(c),
        })
    }

    fn scrape_title(node: &Node) -> String {
        // There are (at least) two formats.
        //
        // 1. Link to a paper or something:
        //
        // ```ignore
        // <h3 class="gs_rt">
        //   <span>
        //       something
        //   </span>
        //   <a href="http://paper.pdf">
        //     Title of paper or something
        //   </a>
        // </h3>
        // ```
        //
        // 'span' may not exists.
        //
        // 2. Not a link:
        //
        // ```ignore
        // <h3 class="gs_rt">
        //   <span>
        //       something
        //   </span>
        //   Title of paper or something
        // </h3>
        // ```

        // 1. Link to a paper or something
        let pos = Class("gs_rt").child(Name("a"));
        if let Some(n) = node.find(pos).nth(0) {
            return n.text();
        }

        // 2. Not a link
        let pos = Class("gs_rt").child(Text);
        node.find(pos)
            .map(|n| n.text())
            .collect::<String>()
            .trim()
            .to_string()
    }

    // Scrape article footer for
    //
    // * cluster id, and
    // * citation count
    fn scrape_id_and_citation(node: &Node) -> Result<(String, u32)> {
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
            parse_id_from_url(id_url).unwrap().to_string()
        };

        let citation_count = parse_citation_count(&citation_node.text())?;

        Ok((id, citation_count))
    }
}

impl Deref for CitationDocument {
    type Target = SearchDocument;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CitationDocument {
    pub fn new(document: Document) -> Self {
        CitationDocument(SearchDocument::new(document))
    }

    pub fn from_read<R: io::Read>(readable: R) -> Result<Self> {
        let document = Document::from_read(readable)?;
        Ok(Self::new(document))
    }

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
            parse_id_from_url(id_url)?.to_string()
        };

        Ok(Paper {
            title,
            id,
            citation_count: None,
        })
    }
}

fn parse_id_from_url(url: &str) -> Result<&str> {
    use regex::Regex;

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(cluster|cites)=(\d+)").unwrap();
    }

    let caps = try_html!(RE.captures(url));
    let id = try_html!(caps.get(2));
    Ok(id.as_str())
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
        assert_eq!(parse_id_from_url("cluster=000000").unwrap(), "000000");
        assert_eq!(
            parse_id_from_url("scholar?cluster=111111").unwrap(),
            "111111"
        );
        assert_eq!(
            parse_id_from_url("scholar?cluster=222222&foo=bar").unwrap(),
            "222222"
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

        assert_eq!(
            papers[0],
            Paper {
                title: String::from("Quantum field theory and critical phenomena"),
                id: String::from("16499695044466828447"),
                citation_count: Some(4821),
            }
        );

        // assert_eq!(
        //     papers[1],
        //     Paper {
        //         title: String::from("Quantum theory of solids"),
        //         id: String::from("8552492368061991976"),
        //         citation_count: Some(4190),
        //     }
        // );

        assert_eq!(
            papers[2],
            Paper {
                title: String::from(
                    "Significance of electromagnetic potentials in the quantum theory"
                ),
                id: String::from("5545735591029960915"),
                citation_count: Some(6961),
            }
        );
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
            Paper {
                title: String::from(
                    "Significance of electromagnetic potentials in the quantum theory"
                ),
                id: String::from("5545735591029960915"),
                citation_count: None,
            }
        );

        assert_eq!(citer_papers.len(), 10);

        assert_eq!(
            citer_papers[0],
            Paper {
                title: String::from("Quantal phase factors accompanying adiabatic changes"),
                id: String::from("15570691018430890829"),
                citation_count: Some(7813),
            }
        );

        assert_eq!(
            citer_papers[1],
            Paper {
                title: String::from("Multiferroics: a magnetic twist for ferroelectricity"),
                id: String::from("9328505180409005573"),
                citation_count: Some(3232),
            }
        );

        assert_eq!(
            citer_papers[2],
            Paper {
                title: String::from("Quantum field theory"),
                id: String::from("14398189842493937255"),
                citation_count: Some(2911),
            }
        );
    }
}
