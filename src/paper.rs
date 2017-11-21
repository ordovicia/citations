use std::fmt;
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paper {
    pub title: String,
    pub id: u64,
    pub citation_count: Option<u32>,
}

impl fmt::Display for Paper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            r#""{}"
    Cluster ID: {}
Citation Count: {}"#,
            self.title,
            self.id,
            citation_count_to_cow(self.citation_count)
        )
    }
}

fn citation_count_to_cow(c: Option<u32>) -> Cow<'static, str> {
    match c {
        Some(c) => c.to_string().into(),
        None => "N/A".into(),
    }
}
