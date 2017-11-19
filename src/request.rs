use reqwest::{self, Url};

use errors::*;

pub const URL_BASE: &str = "https://scholar.google.com/scholar";

pub trait Query {
    fn to_url(&self) -> Result<Url>;
}

pub fn send_request<Q: Query>(query: &Q) -> Result<String> {
    use reqwest::header::UserAgent;

    const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:57.0) Gecko/20100101 Firefox/57.0";

    let client = reqwest::Client::new();
    let url = query.to_url()?;
    let mut res = client.get(url).header(UserAgent::new(USER_AGENT)).send()?;

    let body = res.text()?;
    Ok(body)
}

pub struct SearchQuery {
    count: u32,
    words: Option<String>,
    authors: Option<String>,
    title_only: bool,
}

const DEFAULT_COUNT: u32 = 5;

impl Default for SearchQuery {
    fn default() -> Self {
        SearchQuery {
            count: DEFAULT_COUNT,
            words: None,
            authors: None,
            title_only: false,
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

        let query = format!(
            "as_q={}\
             &as_epq=\
             &as_eq=\
             &as_occt={}\
             &as_sauthors={}\
             &as_publication=\
             &as_ylo=\
             &as_yhi=\
             &as_vis=0\
             &btnG=\
             &hl=en\
             &num={}\
             &as_sdt=0%2C5",
            option_stringify!(self.words),
            if self.title_only { "title" } else { "any" },
            option_stringify!(self.authors),
            self.count,
        );
        url.set_query(Some(&query));

        Ok(url)
    }
}

impl SearchQuery {
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

    pub fn get_count(&self) -> u32 {
        self.count
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::default();
    ///
    /// q.set_words("foo");
    /// assert_eq!(q.get_words(), &Some(String::from("foo")));
    ///
    /// q.set_words("bar");
    /// assert_eq!(q.get_words(), &Some(String::from("bar")));
    /// ```
    pub fn set_words(&mut self, words: &str) {
        self.words = Some(words.to_owned());
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::default();
    /// assert!(q.get_words().is_none());
    ///
    /// q.append_words("foo");
    /// assert_eq!(q.get_words(), &Some(String::from("foo")));
    ///
    /// q.append_words("bar");
    /// assert_eq!(q.get_words(), &Some(String::from("foo bar")));
    /// ```
    pub fn append_words(&mut self, words: &str) {
        match self.words {
            Some(ref mut w) => {
                w.push(' ');
                w.push_str(words);
            }
            None => {
                self.words = Some(words.to_owned());
            }
        }
    }

    pub fn get_words(&self) -> &Option<String> {
        &self.words
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::default();
    ///
    /// q.set_phrase("foo bar");
    /// assert_eq!(q.get_words(), &Some(String::from(r#""foo bar""#)));
    /// ```
    pub fn set_phrase(&mut self, phrase: &str) {
        self.set_words(&format!("\"{}\"", phrase));
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::default();
    ///
    /// q.append_phrase("foo bar");
    /// assert_eq!(q.get_words(), &Some(String::from(r#""foo bar""#)));
    ///
    /// q.append_phrase("baz qux");
    /// assert_eq!(q.get_words(), &Some(String::from(r#""foo bar" "baz qux""#)));
    /// ```
    pub fn append_phrase(&mut self, phrase: &str) {
        self.append_words(&format!("\"{}\"", phrase));
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::default();
    ///
    /// q.set_authors("albert");
    /// assert_eq!(q.get_authors(), &Some(String::from("albert")));
    ///
    /// q.set_authors("einstein");
    /// assert_eq!(q.get_authors(), &Some(String::from("einstein")));
    /// ```
    pub fn set_authors(&mut self, authors: &str) {
        self.authors = Some(authors.to_owned());
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::default();
    ///
    /// q.append_authors("albert");
    /// assert_eq!(q.get_authors(), &Some(String::from("albert")));
    ///
    /// q.append_authors("einstein");
    /// assert_eq!(q.get_authors(), &Some(String::from("albert einstein")));
    /// ```
    pub fn append_authors(&mut self, authors: &str) {
        match self.authors {
            Some(ref mut a) => {
                a.push(' ');
                a.push_str(authors);
            }
            None => {
                self.authors = Some(authors.to_owned());
            }
        }
    }

    pub fn get_authors(&self) -> &Option<String> {
        &self.authors
    }

    /// ```
    /// use scholar::request::SearchQuery;
    ///
    /// let mut q = SearchQuery::default();
    ///
    /// assert_eq!(q.get_title_only(), false);
    ///
    /// q.set_title_only(true);
    /// assert_eq!(q.get_title_only(), true);
    ///
    /// q.set_title_only(false);
    /// assert_eq!(q.get_title_only(), false);
    /// ```
    pub fn set_title_only(&mut self, title_only: bool) {
        self.title_only = title_only;
    }

    pub fn get_title_only(&self) -> bool {
        self.title_only
    }

    fn is_valid(&self) -> bool {
        self.words.is_some() || self.authors.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_query_to_url() {
        let mut q = SearchQuery::default();

        const NEW_COUNT: u32 = DEFAULT_COUNT + 1;
        q.set_count(NEW_COUNT);
        q.set_phrase("quantum theory");
        q.set_authors("albert einstein");
        q.set_title_only(true);

        assert_eq!(
            q.to_url().unwrap(),
            Url::parse(&format!(
                "{}?\
                 as_q=\"quantum theory\"\
                 &as_epq=\
                 &as_eq=\
                 &as_occt=title\
                 &as_sauthors=albert%20einstein\
                 &as_publication=\
                 &as_ylo=\
                 &as_yhi=\
                 &as_vis=0\
                 &btnG=\
                 &hl=en\
                 &num={}\
                 &as_sdt=0%2C5",
                URL_BASE,
                NEW_COUNT
            )).unwrap()
        );
    }

    #[test]
    fn search_query_is_valid_pass() {
        {
            let mut q = SearchQuery::default();

            q.set_words("foo");
            assert!(q.is_valid());
        }

        {
            let mut q = SearchQuery::default();

            q.set_authors("foo");
            assert!(q.is_valid());
        }
    }

    #[test]
    fn search_query_is_valid_fail() {
        let q = SearchQuery::default();
        assert!(!q.is_valid());
    }
}
