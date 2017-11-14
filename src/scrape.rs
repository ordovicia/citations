use regex::Regex;
use select::document::Document;
use select::predicate::{Attr, Name, Predicate};

use super::Paper;
use errors::*;

macro_rules! try_html {
    ($a: expr) => { $a.ok_or(ErrorKind::BadHtml)? }
}

pub fn scrape_target_paper(html: &Document) -> Result<Paper> {
    let node = {
        let pos = Attr("id", "gs_rt_hdr")
            .descendant(Name("h2"))
            .descendant(Name("a"));
        try_html!(html.find(pos).nth(0))
    };

    let name = node.text();

    let id_url = try_html!(node.attr("href"));
    let id = parse_id_from_url(&id_url)?.to_string();

    Ok(Paper { name, id })
}

fn parse_id_from_url(url: &str) -> Result<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(cluster=)(\d+)").unwrap();
    }

    let caps = try_html!(RE.captures(url));
    let id = try_html!(caps.get(2));
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
