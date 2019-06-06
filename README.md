To use vendored sources, add this to your .cargo/config for this project:

    [source.crates-io]
    replace-with = "vendored-sources"

    [source."https://github.com/shadowmint/rust-base-logging"]
    git = "https://github.com/shadowmint/rust-base-logging"
    branch = "master"
    replace-with = "vendored-sources"

    [source.vendored-sources]
    directory = "/Users/doug/dev/now/rust-pup/vendor"