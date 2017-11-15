error_chain!{
    foreign_links {
        Io(::std::io::Error);
        Reqwest(::reqwest::Error);
    }

    errors {
        Cli(e: String) {
            description("CLI usage error")
            display("{}", e)
        }
        BadHtml {
            description("Bad HTML structure")
        }
        InvalidQuery {
            description("Invalid query")
        }
    }
}
