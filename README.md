citations [![Build Status](https://travis-ci.org/ordovicia/citations.svg?branch=master)](https://travis-ci.org/ordovicia/citations)
===============

Scrapes [Google Scholar](https://scholar.google.com) to enumerate papers by which a given paper is cited.

This program does **not** send a request; only analyzes a given HTML file locally.

## Usage
```
$ cargo run file
```

where `file` is a downloaded HTML file of a citation page
(e.g. `https://scholar.google.com/scholar?cites=8174092782678430881&as_sdt=2005&sciodt=0,5&hl=en`).
