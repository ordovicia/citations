error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Serde(::serde_json::Error);
    }

    links {
        Scholar(::scholar::errors::Error, ::scholar::errors::ErrorKind);
    }
}
