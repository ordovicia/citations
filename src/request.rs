use reqwest::{self, Url};

use errors::*;

pub const URL_BASE: &str = "https://scholar.google.com/scholar";

pub trait Query {
    fn to_url(&self) -> Result<Url>;
}

pub fn send_query<Q: Query>(query: &Q) -> Result<String> {
    use reqwest::header::UserAgent;

    const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:57.0) Gecko/20100101 Firefox/57.0";

    let client = reqwest::Client::new();
    let url = query.to_url()?;
    let mut res = client.get(url).header(UserAgent::new(USER_AGENT)).send()?;

    let body = res.text()?;
    Ok(body)
}

const DEFAULT_COUNT: u32 = 5;

pub struct SearchQuery {
    count: u32,
    words: Option<String>,
}

impl Default for SearchQuery {
    fn default() -> Self {
        SearchQuery {
            count: DEFAULT_COUNT,
            words: None,
        }
    }
}

impl Query for SearchQuery {
    fn to_url(&self) -> Result<Url> {
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

        let mut url = Url::parse(URL_BASE).unwrap();

        // scholar?as_q=&as_epq=&as_oq=&as_eq=&as_occt=any&as_sauthors=albert%20einstein&as_publication=&as_ylo=&as_yhi=&as_vis=0&btnG=&hl=en&num=1&as_sdt=0%2C5",

        let query = format!(
            "num={}\
             &q={}",
            self.count,
            option_stringify!(self.words)
        );
        url.set_query(Some(&query));

        Ok(url)
    }
}

impl SearchQuery {
    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::default();
    /// q.set_count(2);
    /// assert_eq!(q.get_count(), 2);
    /// ```
    pub fn set_count(&mut self, count: u32) {
        use std::cmp;

        const MAX_PAGE_RESULTS: u32 = 10;
        self.count = cmp::min(count, MAX_PAGE_RESULTS);
    }

    pub fn get_words(&self) -> &Option<String> {
        &self.words
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::default();
    /// assert!(q.get_words().is_none());
    ///
    /// q.set_words("foo");
    /// assert_eq!(q.get_words(), &Some(String::from("foo")));
    /// ```
    pub fn set_words(&mut self, words: &str) {
        match self.words {
            Some(ref mut w) => {
                w.push_str(words);
            }
            None => {
                self.words = Some(words.to_string());
            }
        }
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::default();
    ///
    /// q.set_phrase("foo bar");
    /// assert_eq!(q.get_words(), &Some(String::from("\"foo bar\"")));
    /// ```
    pub fn set_phrase(&mut self, phrase: &str) {
        self.set_words(&format!("\"{}\"", phrase));
    }

    fn is_valid(&self) -> bool {
        self.words.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_query_to_url_count() {
        let mut q = SearchQuery::default();

        const NEW_COUNT: u32 = DEFAULT_COUNT + 1;
        q.set_count(NEW_COUNT);
        q.set_words("foo");

        assert_eq!(
            q.to_url().unwrap(),
            Url::parse(&format!("{}?num={}&q=foo", URL_BASE, NEW_COUNT)).unwrap()
        );
    }

    #[test]
    fn search_query_to_url_words() {
        let mut q = SearchQuery::default();

        q.set_words("foo bar");

        assert_eq!(
            q.to_url().unwrap(),
            Url::parse(&format!("{}?num={}&q=foo%20bar", URL_BASE, DEFAULT_COUNT)).unwrap()
        );
    }

    #[test]
    fn search_query_to_url_phrase() {
        let mut q = SearchQuery::default();

        q.set_phrase("foo bar");
        assert_eq!(
            q.to_url().unwrap(),
            Url::parse(&format!("{}?num={}&q=\"foo bar\"", URL_BASE, DEFAULT_COUNT)).unwrap()
        );
    }

    #[test]
    fn search_query_is_valid_pass() {
        let mut q = SearchQuery::default();

        q.set_words("foo");
        assert!(q.is_valid());
    }

    #[test]
    fn search_query_is_valid_fail() {
        let q = SearchQuery::default();
        assert!(!q.is_valid());
    }
}
