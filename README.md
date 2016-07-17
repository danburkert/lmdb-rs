[![Build Status](https://travis-ci.org/danburkert/lmdb-rs.svg?branch=master)](https://travis-ci.org/danburkert/lmdb-rs)
[![Windows Build Status](https://ci.appveyor.com/api/projects/status/0bw21yfqsrsv3soh/branch/master?svg=true)](https://ci.appveyor.com/project/danburkert/lmdb-rs/branch/master)

[Documentation](http://danburkert.github.io/lmdb-rs/lmdb/index.html)

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
