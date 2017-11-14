error_chain!{
    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        Cli(e: String) {
            description("CLI usage error")
            display("{}", e)
        }
        BadHtml {
            description("Bad HTML structure")
        }
    }
}
