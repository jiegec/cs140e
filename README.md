cs140e
=======================================

My code for Stanford cs140e. Use `./gendiff.sh` to generate diff from subdirectories and commit into this git repo.

Using `alias code-rust="env RUST_TARGET_PATH=(pwd) RUST_LOG=rls=debug CARGO_INCREMENTAL=0 code-insiders"` in my fish config.

rustup toolchain install nightly-2018-03-1
rustup component add rustfmt-preview rls-preview rust-src rust-analysis
