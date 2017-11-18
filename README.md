scholar [![Build Status](https://travis-ci.org/ordovicia/scholar.svg?branch=master)](https://travis-ci.org/ordovicia/scholar)
=======

Google Scholar scraper written in Rust.

```
USAGE:
    scholar [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --count <count>                Maximum number of results
    -w, --words <words>                Search papers with these words
    -p, --phrase <phrase>              Search papers with this exact phrase
    -a, --authors <authors>            Search papers with these authors
        --search-html <search-html>    HTML file of search results
        --cite-html <cite-html>        HTML file of citers list
```

## Note

**Do not abuse**.
If you send requests too frequently, Google Scholar will block your access temporarily.
Currently, this program outputs nothing when blocked.

## Similar Projects

* Inspired by [ckreibich/scholar.py](https://github.com/ckreibich/scholar.py)
