# lmdbd-rs

Safe Rust bindings for the [Symas Lightning Memory-Mapped Database(LMDB)](http://symas.com/mdb/).

Provides the minimal amount of abstraction necessary to interact with LMDB safely in Rust. In
general, the API is very similar to the LMDB [C-API](http://symas.com/mdb/doc/).

## TODO

* Cursors.
* Zero-copy put API.
* Nested transactions.
* Database statistics.
