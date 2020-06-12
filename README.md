# 📂 tame-gcs

[![Build Status](https://github.com/EmbarkStudios/tame-gcs/workflows/CI/badge.svg)](https://github.com/EmbarkStudios/tame-gcs/actions?workflow=CI)
[![Crates.io](https://img.shields.io/crates/v/tame-gcs.svg)](https://crates.io/crates/tame-gcs)
[![Docs](https://docs.rs/tame-gcs/badge.svg)](https://docs.rs/tame-gcs)
[![Contributor Covenant](https://img.shields.io/badge/contributor%20covenant-v1.4%20adopted-ff69b4.svg)](CODE_OF_CONDUCT.md)
[![Embark](https://img.shields.io/badge/embark-open%20source-blueviolet.svg)](http://embark.games)

`tame-gcs` is a crate with a limited set of [Google Cloud Storage(GCS)](https://cloud.google.com/storage/) operations that follows the [sans-io](https://sans-io.readthedocs.io/) approach.

## Why?

* You want to control how you actually make HTTP requests against GCS.

## Why not?

* This crate only supports some operations.
* There are several other GCS crates available that have many more features and are easier
to work with, if you don't care about what HTTP clients they use.
* This crate requires more boilerplate to work with.

## Examples

The examples directory includes a simplified version of [gsutil](https://cloud.google.com/storage/docs/gsutil). This
is a work in progress that gives examples of using the different operations that are currently supported in this crate.

* [cat](examples/gsutil/cat.rs) - Shows an example [Object::download](https://docs.rs/tame-gcs/latest/tame_gcs/objects/struct.Object.html#method.download)
* [cp](examples/gsutil/cp.rs) - Shows an example of [Object::download](https://docs.rs/tame-gcs/latest/tame_gcs/objects/struct.Object.html#method.download) as well as [Object::insert_multipart](https://docs.rs/tame-gcs/latest/tame_gcs/objects/struct.Object.html#method.insert_multipart)
* [ls](examples/gsutil/ls.rs) - Shows an example of [Object::list](https://docs.rs/tame-gcs/latest/tame_gcs/objects/struct.Object.html#method.list)
* [signurl](examples/gsutil/signurl.rs) - Shows an example of [UrlSigner](https://docs.rs/tame-gcs/latest/tame_gcs/signed_url/struct.UrlSigner.html)
* [stat](examples/gsutil/stat.rs) - Shows an example of [Object::get](https://docs.rs/tame-gcs/latest/tame_gcs/objects/struct.Object.html#method.get)

## Contributing

We welcome community contributions to this project.

Please read our [Contributor Guide](CONTRIBUTING.md) for more information on how to get started.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
