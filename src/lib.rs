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

/// `Error`-related structs defined with `error-chain`.
pub mod errors;

/// `Paper` struct.
pub mod paper;

/// Send requests to Google Scholar.
pub mod request;

/// Scrape HTML document to get information of papers.
pub mod scrape;
