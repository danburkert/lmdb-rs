[![Build Status](https://travis-ci.org/danburkert/lmdb-rs.svg?branch=master)](https://travis-ci.org/danburkert/lmdb-rs)

[Documentation](http://rust-ci.org/danburkert/lmdb-rs/doc/lmdb/)

[Cargo](https://crates.io/crates/lmdb)

# lmdb-rs

Idiomatic and safe APIs for interacting with the
[Symas Lightning Memory-Mapped Database (LMDB)](http://symas.com/mdb/).

## Building from Source

```bash
git clone --recursive git@github.com:danburkert/lmdb-rs.git
cd lmdb-rs
cargo build
```

## TODO

* [x] lmdb-sys.
* [x] Cursors.
* [x] Zero-copy put API.
* [ ] Nested transactions.
* [ ] Database statistics.
