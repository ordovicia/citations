use std::ops::Deref;
use regex::Regex;

use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Class, Name, Predicate};

use paper::Paper;
use errors::*;

macro_rules! try_option_html {
    ($a: expr) => { $a.ok_or(ErrorKind::BadHtml)? }
}

pub struct PapersDocument(Document);
pub struct SearchDocument(PapersDocument);
pub struct CitingDocument(PapersDocument);

impl Deref for PapersDocument {
    type Target = Document;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PapersDocument {
    pub fn scrape_papers(&self) -> Result<Vec<Paper>> {
        let nodes = self.find(Attr("id", "gs_res_ccl_mid").descendant(Class("gs_ri")));

        let mut papers = Vec::with_capacity(10);
        for n in nodes {
            papers.push(Self::scrape_paper_one(&n)?);
        }

        Ok(papers)
    }

    fn scrape_paper_one(node: &Node) -> Result<Paper> {
        let title = {
            let pos = Class("gs_rt").descendant(Name("a"));
            let n = try_option_html!(node.find(pos).nth(0));
            n.text()
        };

        let id = {
            let pos = Class("gs_fl").descendant(Class("gs_nph"));
            let n = try_option_html!(node.find(pos).nth(1));
            let id_url = try_option_html!(n.attr("href"));
            parse_id_from_url(&id_url)?.to_string()
        };

        Ok(Paper { title, id })
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

impl Deref for CitingDocument {
    type Target = PapersDocument;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CitingDocument {
    pub fn new(document: Document) -> Self {
        CitingDocument(PapersDocument(document))
    }

    pub fn scrape_target_paper(&self) -> Result<Paper> {
        let node = {
            let pos = Attr("id", "gs_rt_hdr")
                .descendant(Name("h2"))
                .descendant(Name("a"));
            try_option_html!(self.find(pos).nth(0))
        };

        let title = node.text();

        let id_url = try_option_html!(node.attr("href"));
        let id = parse_id_from_url(&id_url)?.to_string();

        Ok(Paper { title, id })
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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_eq_result {
        ($result: expr, $expected: expr) => {
            assert!($result.is_ok());
            assert_eq!($result.unwrap(), $expected);
        }
    }

    #[test]
    fn parse_id_from_url_pass() {
        assert_eq_result!(parse_id_from_url("cluster=000000"), "000000");
        assert_eq_result!(parse_id_from_url("scholar?cluster=111111"), "111111");
        assert_eq_result!(
            parse_id_from_url("scholar?cluster=222222&foo=bar"),
            "222222"
        );
    }

    #[test]
    fn parse_id_from_url_fail() {
        assert!(parse_id_from_url("foo").is_err());
        assert!(parse_id_from_url("claster=000000").is_err());
        assert!(parse_id_from_url("cluster=aaaaaa").is_err());
    }
}
