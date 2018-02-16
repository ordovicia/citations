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
        //     each paper
        //   </div>
        //   <div class="gs_ri">
        //     each paper
        //   </div>
        //   ...
        // </div>

        let paper_nodes = {
            let pos = Attr("id", "gs_res_ccl_mid").descendant(Class("gs_ri"));
            self.find(pos)
        };

        let mut papers = Vec::with_capacity(10);
        for n in paper_nodes {
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
                let doc = Document::from(s);
                $struct(doc)
            }
        }

        impl $struct {
            pub fn new(doc: Document) -> Self {
                $struct(doc)
            }

            // like Document::from_read()
            pub fn from_read<R: io::Read>(readable: R) -> Result<Self> {
                let doc = Document::from_read(readable)?;
                Ok(Self::new(doc))
            }
        }
    }
}

macro_rules! try_html_bad {
    ($a: expr) => { $a.ok_or(ErrorKind::BadHtml)? }
}

macro_rules! try_html_found {
    ($a: expr) => { $a.ok_or(ErrorKind::ResultNotFount)? }
}

pub struct SearchDocument(Document);
impl_from_to_document!(SearchDocument);

pub struct CitationDocument(Document);
impl_from_to_document!(CitationDocument);

impl CitationDocument {
    pub fn scrape_target_paper_with_citers(&self) -> Result<Paper> {
        let target_paper = self.scrape_target_paper()?;
        let citers = self.scrape_papers()?;

        let mut paper = target_paper;
        paper.citers = Some(citers);
        Ok(paper)
    }

    fn scrape_target_paper(&self) -> Result<Paper> {
        // <div id="gs_rt_hdr">
        //   <h2>
        //     <a href="https://scholar.google.co.jp/scholar?cluster=0">
        //       title
        //     </a>
        //   </h2>
        //   something
        // </div>

        let target_paper_node = {
            let pos = Attr("id", "gs_rt_hdr")
                .child(Name("h2"))
                .child(Name("a").or(Text));
            try_html_found!(self.find(pos).nth(0))
        };

        let title = target_paper_node.text();
        let cluster_id = {
            let id_url = try_html_bad!(target_paper_node.attr("href"));
            parse_cluster_id(id_url)?
        };

        Ok(Paper::new(&title, cluster_id))
    }
}

pub struct ClusterDocument(Document);
impl_from_to_document!(ClusterDocument);

impl ClusterDocument {
    pub fn scrape_target_paper(&self) -> Result<Paper> {
        let paper_node = {
            let pos = Attr("id", "gs_res_ccl_mid").descendant(Class("gs_ri"));
            try_html_found!(self.find(pos).nth(0))
        };
        scrape_paper_one(&paper_node)
    }
}

struct ArticleTitle {
    title: String,
    link: Option<String>,
}

struct ArticleHeader {
    year: Option<u32>,
}

struct ArticleFooter {
    cluster_id: u64,
    citation_count: u32,
}

fn scrape_paper_one(node: &Node) -> Result<Paper> {
    let ArticleTitle { title, link } = scrape_article_title(node);
    let ArticleHeader { year } = scrape_article_header(node);
    let ArticleFooter {
        cluster_id,
        citation_count,
    } = scrape_article_footer(node)?;

    let mut paper = Paper::new(&title, cluster_id);
    paper.link = link;
    paper.year = year;
    paper.citation_count = Some(citation_count);

    Ok(paper)
}

fn scrape_article_title(node: &Node) -> ArticleTitle {
    // There are (at least) two formats.
    //
    // 1. Link to a paper or something:
    //
    // <h3 class="gs_rt">
    //   <span>
    //     something
    //   </span>
    //   <a href="http://paper.pdf">
    //     title of paper or something
    //   </a>
    // </h3>
    //
    // 'span' may not exists.
    //
    // 2. Not a link:
    //
    // <h3 class="gs_rt">
    //   <span>
    //     something
    //   </span>
    //   title of paper or something
    // </h3>

    if let Some(n) = {
        let pos = Class("gs_rt").child(Name("a"));
        node.find(pos).nth(0)
    } {
        // 1. Link to a paper or something
        ArticleTitle {
            title: n.text(),
            link: n.attr("href").map(ToOwned::to_owned),
        }
    } else {
        // 2. Not a link
        let children = {
            let pos = Class("gs_rt");
            node.find(pos).into_selection().children()
        };
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
        ArticleTitle {
            title: concated_text,
            link: None,
        }
    }
}

fn scrape_article_header(node: &Node) -> ArticleHeader {
    // There are (at least) two formats for publishment information:
    //
    // 1. with journal etc. at the third part:
    //
    // <div class="gs_a">
    //   author - journal etc., year - journal etc.
    // </div>
    //
    // 'journal etc.' at the second part may be ommited.
    //
    // 2. only two parts:
    //
    // <div class="gs_a">
    //   author - journal etc., year
    // </div>
    //
    // 'journal etc.' at the second part may be ommited.
    //
    //
    // Author and 'journal etc.' can be a link:
    //
    // <div class="gs_a">
    //   <a href="/citations?user=0">author</a> - journal etc., year - journal etc.
    // </div>

    let year_node = {
        let pos = Class("gs_a").descendant(Text);
        node.find(pos)
            .into_selection()
            .filter(|n: &Node| parse_year(&n.text()).is_ok())
            .first()
    };
    let year = year_node.map(|n| parse_year(&n.text()).unwrap());

    ArticleHeader { year }
}

fn parse_year(text: &str) -> Result<u32> {
    use regex::Regex;

    lazy_static! {
        static ref RE: Regex = Regex::new(r".*\s-\s.*((18|19|20)(\d{2}))(\s-\s.+)?").unwrap();
    }

    let year = {
        let caps = try_html_bad!(RE.captures(text));
        let year = try_html_bad!(caps.get(1));
        year.as_str().parse().unwrap()
    };

    Ok(year)
}

fn scrape_article_footer(node: &Node) -> Result<ArticleFooter> {
    // Footer format:
    //
    // <div class="gs_fl">
    //   something
    //   <a href="/scholar?cites=000000>Cited by 999</a>
    //   something
    // </div>

    let footer_nodes = {
        let pos = Class("gs_fl");
        try_html_bad!(node.find(pos).nth(0))
            .children()
            .into_selection()
    };

    let citation_node = footer_nodes
        .filter(|n: &Node| {
            if let Some(id_url) = n.attr("href") {
                parse_cluster_id(id_url).is_ok()
            } else {
                false
            }
        })
        .first();
    let citation_node = try_html_bad!(citation_node);

    let cluster_id = {
        let id_url = citation_node.attr("href").unwrap();
        parse_cluster_id(id_url).unwrap()
    };
    let citation_count = parse_citation_count(&citation_node.text())?;

    Ok(ArticleFooter {
        cluster_id,
        citation_count,
    })
}

fn parse_cluster_id(url: &str) -> Result<u64> {
    use regex::Regex;

    lazy_static! {
        static ref RE: Regex = Regex::new(r"(cluster|cites)=(\d+)").unwrap();
    }

    let cluster_id = {
        let caps = try_html_bad!(RE.captures(url));
        let id = try_html_bad!(caps.get(2));
        id.as_str().parse()?
    };

    Ok(cluster_id)
}

fn parse_citation_count(text: &str) -> Result<u32> {
    use regex::Regex;

    lazy_static! {
        static ref RE: Regex = Regex::new(r"[^\d]+(\d+)").unwrap();
    }

    let count = {
        let caps = try_html_bad!(RE.captures(text));
        let count = try_html_bad!(caps.get(1));
        count.as_str().parse().unwrap()
    };

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_year_pass() {
        assert_eq!(parse_year("foo - journal, 2000 - bar").unwrap(), 2000);
        assert_eq!(parse_year("foo bar - 1999 - baz, qrux").unwrap(), 1999);
        assert_eq!(parse_year("foo - journal, 1998").unwrap(), 1998);
        assert_eq!(parse_year("foo - 1899").unwrap(), 1899);
        assert_eq!(parse_year(" - journal, 1898").unwrap(), 1898);
        assert_eq!(parse_year(" - 1800").unwrap(), 1800);
    }

    #[test]
    fn parse_year_fail() {
        assert!(parse_year("foo - journal - bar").is_err());
        assert!(parse_year("foo - journal").is_err());
        assert!(parse_year("- journal, 1898").is_err());
        assert!(parse_year("- 1800").is_err());
    }

    #[test]
    fn parse_cluster_id_pass() {
        assert_eq!(parse_cluster_id("cluster=123456").unwrap(), 123456);
        assert_eq!(parse_cluster_id("scholar?cluster=654321").unwrap(), 654321);
        assert_eq!(
            parse_cluster_id("scholar?cluster=222222&foo=bar").unwrap(),
            222222
        );
    }

    #[test]
    fn parse_cluster_id_fail() {
        assert!(parse_cluster_id("foo").is_err());
        assert!(parse_cluster_id("claster=000000").is_err());
        assert!(parse_cluster_id("cluster=aaaaaa").is_err());
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
            paper.year = Some(1996);
            paper.citation_count = Some(4821);
            paper
        });

        assert_eq!(papers[1], {
            let mut paper = Paper::new("Quantum theory of solids", 8552492368061991976);
            paper.year = Some(1963);
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
            paper.year = Some(1959);
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
            paper.year = Some(1984);
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
            paper.year = Some(2007);
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
            paper.year = Some(1996);
            paper.citation_count = Some(2911);
            paper
        });
    }

    #[test]
    fn cluster_document_scrape_test() {
        // TODO
    }
}
