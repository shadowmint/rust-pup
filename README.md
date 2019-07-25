# pup

Pup is generic task execution engine.

Pup is *not* a dependency resolution system or an eventual consistency tool;
it just runs the tasks in the manifest one by one.

## build

Use the `build.sh` script.

## vendored build

To use vendored sources, add this to your .cargo/config for this project:

    [source.crates-io]
    replace-with = "vendored-sources"

    [source."https://github.com/shadowmint/rust-base-logging"]
    git = "https://github.com/shadowmint/rust-base-logging"
    branch = "master"
    replace-with = "vendored-sources"

    [source.vendored-sources]
    directory = "/Users/doug/dev/now/rust-pup/vendor"
