//! `Paper` struct.

use std::fmt;
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Paper {
    pub title: String,
    /// Link to PDF, HTML, etc.
    pub link: Option<String>,
    /// Cluster ID of paper.
    pub cluster_id: u64,
    pub citation_count: Option<u32>,
    pub citers: Option<Vec<Paper>>,
    /// URL of citation list page of Google Scholar.
    pub citation_url: String,
}

impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#""{}"
 Link to paper: {}
    Cluster ID: {}
Citation count: {}
 Citation List: {}"#,
            self.title,
            option_na(&self.link),
            self.cluster_id,
            option_na(&self.citation_count.map(|u| u.to_string())),
            self.citation_url,
        )
    }
}

impl Paper {
    /// Create new `Paper` with specified `title` and `cluster_id`.
    /// `citation_url` is set according to `cluster_id`.
    /// `citation_count` and `citers` are left `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use scholar::paper::Paper;
    ///
    /// let paper = Paper::new("foo", 42);
    ///
    /// assert_eq!(paper,
    ///     Paper {
    ///         title: String::from("foo"),
    ///         link: None,
    ///         cluster_id: 42,
    ///         citation_count: None,
    ///         citers: None,
    ///         citation_url: format!("https://scholar.google.com/scholar?cites={}", 42),
    ///     });
    /// ```
    pub fn new(title: &str, cluster_id: u64) -> Self {
        let title = title.to_owned();
        let citation_url = Self::cluster_id_to_citation_url(cluster_id);

        Self {
            title,
            link: None,
            cluster_id,
            citation_count: None,
            citers: None,
            citation_url,
        }
    }

    fn cluster_id_to_citation_url(id: u64) -> String {
        format!("{}?cites={}", super::GOOGLESCHOLAR_URL_BASE, id)
    }
}

fn option_na(c: &Option<String>) -> Cow<'static, str> {
    match *c {
        Some(ref c) => c.clone().into(),
        None => "N/A".into(),
    }
}
