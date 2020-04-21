#! /bin/sh
cargo install systemfd cargo-watch
systemfd --no-pid -s http::8080 -- cargo watch -x run
