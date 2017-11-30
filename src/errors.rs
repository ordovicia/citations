//! `Error`-related structs defined with `error-chain`.

error_chain!{
    foreign_links {
        Io(::std::io::Error);
        Reqwest(::reqwest::Error);
        Parse(::std::num::ParseIntError);
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
