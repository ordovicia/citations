#[derive(Debug)]
pub struct Paper {
    pub title: String,
    pub id: String,
    pub citation_count: Option<u32>,
}
