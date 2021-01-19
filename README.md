# 📂 tame-gcs

[![Embark](https://img.shields.io/badge/embark-open%20source-blueviolet.svg)](https://embark.dev)
[![Embark](https://img.shields.io/badge/discord-ark-%237289da.svg?logo=discord)](https://discord.gg/dAuKfZS)
[![Crates.io](https://img.shields.io/crates/v/tame-gcs.svg)](https://crates.io/crates/tame-gcs)
[![Docs](https://docs.rs/tame-gcs/badge.svg)](https://docs.rs/tame-gcs)
[![dependency status](https://deps.rs/repo/github/EmbarkStudios/tame-gcs/status.svg)](https://deps.rs/repo/github/EmbarkStudios/tame-gcs)
[![Build Status](https://github.com/EmbarkStudios/tame-gcs/workflows/CI/badge.svg)](https://github.com/EmbarkStudios/tame-gcs/actions?workflow=CI)

`tame-gcs` is a crate with a limited set of [Google Cloud Storage(GCS)](https://cloud.google.com/storage/) operations that follows the [sans-io](https://sans-io.readthedocs.io/) approach.

## Why?

* You want to control how you actually make HTTP requests against GCS.
* You want to have more control over your dependencies, and not be bottlenecked for sticking to a particular version, or quickly upgrading, your HTTP related crates.

## Why not?

* This crate only supports some operations.
* There are several other GCS crates available that have many more features and are easier to work with, if you don't care about what HTTP clients they use.
* This crate requires more boilerplate to work with.

## Example

For example usage, see the [gsutil](https://github.com/EmbarkStudios/gsutil) crate, which reimplements parts of the official gsutil tool.

## Contributing

[![Contributor Covenant](https://img.shields.io/badge/contributor%20covenant-v1.4-ff69b4.svg)](../CODE_OF_CONDUCT.md)

We welcome community contributions to this project.

Please read our [Contributor Guide](CONTRIBUTING.md) for more information on how to get started.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
