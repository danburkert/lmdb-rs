[![Build Status](https://travis-ci.org/danburkert/lmdb-rs.svg?branch=master)](https://travis-ci.org/danburkert/lmdb-rs)

[Documentation](http://rust-ci.org/danburkert/lmdb-rs/doc/lmdb/)

[Cargo](https://crates.io/crates/lmdb)

# lmdb-rs

Safe Rust bindings for the [Symas Lightning Memory-Mapped Database (LMDB)](http://symas.com/mdb/).

Provides the minimal amount of abstraction necessary to interact with LMDB safely in Rust. In
general, the API is very similar to the LMDB [C-API](http://symas.com/mdb/doc/).

## Building from Source

```bash
git clone --recursive git@github.com:danburkert/lmdb-rs.git
cd lmdb-rs
cargo build
```

## TODO

* [x] lmdb-sys.
* [x] Cursors.
* [ ] Zero-copy put API.
* [ ] Nested transactions.
* [ ] Database statistics.
