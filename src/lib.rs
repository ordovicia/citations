//! Google Scholar scraper.

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate reqwest;
extern crate select;
extern crate serde;
#[macro_use]
extern crate serde_derive;

pub mod errors;
pub mod paper;
pub mod request;
pub mod scrape;

const GOOGLESCHOLAR_URL_BASE: &str = "https://scholar.google.com/scholar";

pub const MAX_RESULT_COUNT: u32 = 10;
