#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate reqwest;
extern crate select;

pub mod errors;
pub mod paper;
pub mod request;
pub mod scrape;
