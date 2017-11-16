use std::io;
use std::ops::Deref;

use regex::Regex;
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Class, Name, Predicate, Text};

use paper::Paper;
use errors::*;

macro_rules! try_option_html {
    ($a: expr) => { $a.ok_or(ErrorKind::BadHtml)? }
}

pub struct PapersDocument(Document);
pub struct SearchDocument(PapersDocument);
pub struct CitersDocument(PapersDocument);

impl Deref for PapersDocument {
    type Target = Document;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PapersDocument {
    pub fn scrape_papers(&self) -> Result<Vec<Paper>> {
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
    fn scrape_title(node: &Node) -> String {
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
    //
    // Format:
    //
    // <div class="gs_fl">
    //   something
    //   <a href="/scholar?cites=000000>Cited by 999</a>
    //   something
    // </div>
    fn scrape_id_and_citation(node: &Node) -> Result<(String, u32)> {
        let pos = Class("gs_fl");
        let footers = try_option_html!(node.find(pos).nth(0)).children();

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
        let citation_node = try_option_html!(citation_node);

        let id = {
            let id_url = citation_node.attr("href").unwrap();
            parse_id_from_url(id_url).unwrap().to_string()
        };

        let citation_count = parse_citation_count(&citation_node.text())?;

        Ok((id, citation_count))
    }
}

impl Deref for SearchDocument {
    type Target = PapersDocument;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> From<&'a str> for SearchDocument {
    fn from(s: &str) -> Self {
        let document = Document::from(s);
        SearchDocument(PapersDocument(document))
    }
}

impl SearchDocument {
    pub fn from_read<R: io::Read>(readable: R) -> Result<Self> {
        let document = Document::from_read(readable)?;
        Ok(SearchDocument(PapersDocument(document)))
    }
}

impl Deref for CitersDocument {
    type Target = PapersDocument;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CitersDocument {
    pub fn new(document: Document) -> Self {
        CitersDocument(PapersDocument(document))
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
            try_option_html!(self.find(pos).nth(0))
        };

        let title = node.text();
        let id = {
            let id_url = try_option_html!(node.attr("href"));
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
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(cluster|cites)=(\d+)").unwrap();
    }

    let caps = try_option_html!(RE.captures(url));
    let id = try_option_html!(caps.get(2));
    Ok(id.as_str())
}

fn parse_citation_count(text: &str) -> Result<u32> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"[^\d]+(\d+)").unwrap();
    }

    let caps = try_option_html!(RE.captures(text));
    let count = {
        let count = try_option_html!(caps.get(1));
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
}
