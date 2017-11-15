error_chain!{
    foreign_links {
        Num(::std::num::ParseIntError);
        Io(::std::io::Error);
        Reqwest(::reqwest::Error);
    }

    errors {
        BadHtml {
            description("Bad HTML structure")
        }
        InvalidQuery {
            description("Invalid query")
        }
    }
}
