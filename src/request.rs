use std::cmp;

use reqwest::{Client, Url};
use reqwest::header::UserAgent;

use errors::*;

pub trait Query {
    fn set_query(&self, url: &mut Url) -> Result<()>;
}

const DEFAULT_COUNT: u32 = 5;

pub struct SearchQuery {
    count: u32,
    words: Option<String>,
}

pub fn send_query<Q: Query>(query: &Q) -> Result<String> {
    const URL_BASE: &str = "https://scholar.google.com/scholar";
    const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:57.0) Gecko/20100101 Firefox/57.0";

    let mut url = Url::parse(URL_BASE).unwrap();
    query.set_query(&mut url)?;

    let client = Client::new();
    let mut res = client.get(url).header(UserAgent::new(USER_AGENT)).send()?;

    let body = res.text()?;
    Ok(body)
}

impl Query for SearchQuery {
    fn set_query(&self, url: &mut Url) -> Result<()> {
        if !self.is_valid() {
            return Err(ErrorKind::InvalidQuery.into());
        }

        macro_rules! option_stringify {
            ($x: expr) => {
                match $x {
                    Some(ref y) => y,
                    None => "",
                }
            }
        }

        // scholar?as_q=&as_epq=&as_oq=&as_eq=&as_occt=any&as_sauthors=albert%20einstein&as_publication=&as_ylo=&as_yhi=&as_vis=0&btnG=&hl=en&num=1&as_sdt=0%2C5",

        let query = format!(
            "num={}\
             &q={}",
            self.count,
            option_stringify!(self.words)
        );
        url.set_query(Some(&query));

        Ok(())
    }
}

impl SearchQuery {
    pub fn new() -> Self {
        SearchQuery {
            count: DEFAULT_COUNT,
            words: None,
        }
    }

    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::new();
    /// assert_eq!(q.get_count(), 1);
    ///
    /// q.set_count(2);
    /// assert_eq!(q.get_count(), 2);
    /// ```
    pub fn set_count(&mut self, count: u32) {
        const MAX_PAGE_RESULTS: u32 = 10;
        self.count = cmp::min(count, MAX_PAGE_RESULTS);
    }

    pub fn get_words(&self) -> &Option<String> {
        &self.words
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::new();
    /// assert!(q.get_words().is_none());
    ///
    /// q.set_words(String::from("foo"));
    /// assert_eq!(q.get_words(), &Some(String::from("foo")));
    /// ```
    pub fn set_words(&mut self, words: String) {
        match self.words {
            Some(ref mut w) => {
                w.push_str(&words);
            }
            None => {
                self.words = Some(words);
            }
        }
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::new();
    ///
    /// q.set_phrase(String::from("foo bar"));
    /// assert_eq!(q.get_words(), &Some(String::from("\"foo bar\"")));
    /// ```
    pub fn set_phrase(&mut self, phrase: String) {
        self.set_words(format!("\"{}\"", phrase));
    }

    fn is_valid(&self) -> bool {
        self.words.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_query_set_query_count() {
        let mut q = SearchQuery::new();
        let mut url = Url::parse("https://example.com").unwrap();

        const NEW_COUNT: u32 = DEFAULT_COUNT + 1;
        q.set_count(NEW_COUNT);
        q.set_words(String::from("foo"));
        q.set_query(&mut url).unwrap();
        assert_eq!(
            url,
            Url::parse(&format!("https://example.com/?num={}&q=foo", NEW_COUNT)).unwrap()
        );
    }

    #[test]
    fn search_query_set_query_words() {
        let mut q = SearchQuery::new();
        let mut url = Url::parse("https://example.com").unwrap();

        q.set_words(String::from("foo bar"));
        q.set_query(&mut url).unwrap();
        assert_eq!(
            url,
            Url::parse(&format!(
                "https://example.com/?num={}&q=foo%20bar",
                DEFAULT_COUNT
            )).unwrap()
        );
    }

    #[test]
    fn search_query_set_query_phrase() {
        let mut q = SearchQuery::new();
        let mut url = Url::parse("https://example.com").unwrap();

        q.set_phrase(String::from("foo bar"));
        q.set_query(&mut url).unwrap();
        assert_eq!(
            url,
            Url::parse(&format!(
                "https://example.com/?num={}&q=\"foo bar\"",
                DEFAULT_COUNT
            )).unwrap()
        );
    }

    #[test]
    fn search_query_is_valid_pass() {
        let mut q = SearchQuery::new();

        q.set_words(String::from("foo"));
        assert!(q.is_valid());
    }

    #[test]
    fn search_query_is_valid_fail() {
        let q = SearchQuery::new();
        assert!(!q.is_valid());
    }
}
