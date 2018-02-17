error_chain! {
    foreign_links {
        Clap(::clap::Error);
        Io(::std::io::Error);
        Serde(::serde_json::Error);
    }

    links {
        Scholar(::scholar::errors::Error, ::scholar::errors::ErrorKind);
    }

    errors {
        Blocked {
            description("Request blocked")
        }
    }
}
