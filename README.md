[WIP] citations
===============

Scrapes [Google Scholar](https://scholar.google.com) to enumerate papers by which a given paper is cited.

This program does *not* request a web page; only analyzes a given HTML locally.

## Usage
```
$ cargo run file
```

where `file` is an HTML downloaded from a citation page
(e.g. `https://scholar.google.com/scholar?cites=8174092782678430881&as_sdt=2005&sciodt=0,5&hl=en`).
