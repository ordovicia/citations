use std::fmt;
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Paper {
    pub title: String,
    pub id: u64,
    pub citation_count: Option<u32>,
    pub citers: Option<Vec<Paper>>,
    pub citation_url: String,
}

impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#""{}"
    Cluster ID: {}
Citation count: {}
  Citation URL: {}"#,
            self.title,
            self.id,
            citation_count_to_cow(self.citation_count),
            self.citation_url,
        )
    }
}

impl Paper {
    pub fn new(title: &str, id: u64) -> Self {
        let title = title.to_owned();
        let citation_url = Self::id_to_citation_url(id);

        Self {
            title,
            id,
            citation_count: None,
            citers: None,
            citation_url,
        }
    }

    fn id_to_citation_url(id: u64) -> String {
        format!("{}?cites={}", super::SCHOLAR_URL_BASE, id)
    }
}

fn citation_count_to_cow(c: Option<u32>) -> Cow<'static, str> {
    match c {
        Some(c) => c.to_string().into(),
        None => "N/A".into(),
    }
}
