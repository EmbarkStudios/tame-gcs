<!-- markdownlint-disable blanks-around-headings blanks-around-lists no-duplicate-heading -->

# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->
## [Unreleased] - ReleaseDate
## [0.11.3] - 2022-02-24
### Added
- [PR#54](https://github.com/EmbarkStudios/tame-gcs/pull/54) added support for [resumable uploads](https://cloud.google.com/storage/docs/resumable-uploads). Thanks [@yottabytt](https://github.com/yottabytt)!

## [0.11.2] - 2022-02-09
### Fixed
- [PR#55](https://github.com/EmbarkStudios/tame-gcs/pull/55) fixed a bug in signed url creation caused by a stray character in the timestamp string.

## [0.11.1] - 2022-02-02
### Fixed
- [PR#53](https://github.com/EmbarkStudios/tame-gcs/pull/53) fixed a bug in deserialization introduced by [PR#52](https://github.com/EmbarkStudios/tame-gcs/pull/52).

## [0.11.0] - 2022-02-02
### Added
- [PR#51](https://github.com/EmbarkStudios/tame-gcs/pull/51) implemented `futures_util::Stream` for `Multipart<Bytes>`. Thanks [@shikhar](https://github.com/shikhar)!

### Changed
- [PR#52](https://github.com/EmbarkStudios/tame-gcs/pull/52) replaced `chrono` with `time` due to maintenance issues with `chrono`.

## [0.10.0] - 2021-03-26
### Changed
- Renamed `Error::API` => `Error::Api` and `Error::SingingError` => `Error::Signing`.
- Inner error details for all `Error` variants are now publicly exposed.

### Added
- [PR#47](https://github.com/EmbarkStudios/tame-gcs/pull/47) added support for the [`object::rewrite`](https://cloud.google.com/storage/docs/json_api/v1/objects/rewrite) operation.

## [0.9.1] - 2021-01-18
### Changed
- Updated `base64` to `0.13`, aligning with the version used by `rustls`

## [0.9.0] - 2021-01-09
### Changed
- Updated bytes to 1.0
- Updated url to 2.2

## [0.8.1] - 2020-11-18
### Fixed
- [PR#36](https://github.com/EmbarkStudios/tame-gcs/pull/36) fixed an issue with the minor version bump of ring from 0.16.15 => 0.16.16.

## [0.8.0] - 2020-10-21
### Added
- Resolved [#30](https://github.com/EmbarkStudios/tame-gcs/issues/30) by deriving `Copy, Clone, Debug, PartialEq, Eq` for `Scopes`, `DigestAlgorithm`, `SigningAlgorithm`, `StorageClass`, `PredefinedAcl`, and `PredefinedAcl`

### Changed
- Updated pin-utils to 0.1.0

## [0.7.3] - 2020-08-19
### Fixed
- Fixed `Object::patch` to not nuke all of your object's metadata and instead do what it was supposed to in the first place.

## [0.7.2] - 2020-06-09
### Added
- Added `Object::patch` for updating metadata for an object.

## [0.7.1] - 2020-06-04
### Changed
- Updated dependencies

## [0.7.0] - 2020-04-15
### Added
- Added `impl<B: AsyncRead + Unpin> AsyncRead for Multipart<B>`. This is gated behind the new `async-multipart` feature. Thanks [@yiwu-arbug](https://github.com/yiwu-arbug)!

## [0.6.1] - 2020-01-21
### Changed
- Updated dependencies
- Made the `gsutil` example use `async`

## [0.6.0] - 2019-12-20
### Changed
- Upgraded `bytes` to `0.5.3`
- Upgraded `http` to `0.2.0`

## [0.5.2] - 2019-12-05
### Added
- Added `content_encoding` to `objects::Metadata`

### Changed
- Updated dependencies

## [0.5.1] - 2019-10-18
### Added
- Added `content_disposition` to `objects::Metadata`

## [0.5.0] - 2019-10-10
### Changed
- Update dependencies
- Replace use of `failure` with `thiserror` for the library
- Replace use of `failure` with `anyhow` in the examples

## [0.4.1] - 2019-08-02
### Added
- Added `ls` example to `gsutil`
- Fleshed out documentation

## [0.4.0] - 2019-08-01
### Added
- Added `cp` and `stat` examples to `gsutil`
- Added `Object::multipart_insert` and corresponding `Multipart<B>` to support multipart uploads

### Changed
- Renamed `ObjectMetadata` to `Metadata` as it is already inside the `objects` module
- Renamed the various `*ObjectResponse` types to just `*Response` as they are in the `objects` module
- Skip serialization of most fields for `objects::Metadata`

## [0.3.4] - 2019-07-22
### Fixed
- Fixed handling of empty `Object::list` responses

## [0.3.3] - 2019-07-22
### Fixed
- Fixed encoding of object paths in `Object::insert`

## [0.3.2] - 2019-07-22
### Fixed
- Fixed signature of `Object::delete`

## [0.3.1] - 2019-07-19
### Added
- Added `Object::list`

## [0.3.0] - 2019-07-17
### Added
- Added `UrlSigner` for generating signed URLs
- Added `signing` feature which implements the components needed for `UrlSigner` to work via `ring`
- Added `StandardQueryParameters`, `Conditionals`, `StorageClass`, `PredefinedAcl`, `Projection`
- Added `gsutil` example CLI
- Added the `cat` and `signurl` examples to `gsutil`

### Changed
- Moved `Object` under `v1`
- Split `Object` methods into separate files
- Renamed `Object::insert` to `insert_simple`

## [0.2.0] - 2019-07-08
### Added
- Added error::Error for consolidating errors from tame-gcs
- Added tame-gcs::Scopes to provide typesafe access to the oauth scopes required by GCS operations
- Added `Object::insert`, `Object::download`, `Object::get`, Object::delete`
- Added `ObjectMetadata` for de/serializing metadata about objects
- Added `ObjectName` and `BucketName` for validating GCS constraints

## [0.1.0] - 2019-07-08
### Added
- Initial add of `tame-gcs`

<!-- next-url -->
[Unreleased]: https://github.com/EmbarkStudios/tame-gcs/compare/0.11.3...HEAD
[0.11.3]: https://github.com/EmbarkStudios/tame-gcs/compare/0.11.2...0.11.3
[0.11.2]: https://github.com/EmbarkStudios/tame-gcs/compare/0.11.1...0.11.2
[0.11.1]: https://github.com/EmbarkStudios/tame-gcs/compare/0.11.0...0.11.1
[0.11.0]: https://github.com/EmbarkStudios/tame-gcs/compare/0.10.0...0.11.0
[0.10.0]: https://github.com/EmbarkStudios/tame-gcs/compare/0.9.1...0.10.0
[0.9.1]: https://github.com/EmbarkStudios/tame-gcs/compare/0.9.0...0.9.1
[0.9.0]: https://github.com/EmbarkStudios/tame-gcs/compare/0.8.1...0.9.0
[0.8.1]: https://github.com/EmbarkStudios/tame-gcs/compare/0.8.0...0.8.1
[0.8.0]: https://github.com/EmbarkStudios/tame-gcs/compare/0.7.3...0.8.0
[0.7.3]: https://github.com/EmbarkStudios/tame-gcs/compare/0.7.2...0.7.3
[0.7.2]: https://github.com/EmbarkStudios/tame-gcs/compare/0.7.1...0.7.2
[0.7.1]: https://github.com/EmbarkStudios/tame-gcs/compare/0.7.0...0.7.1
[0.7.0]: https://github.com/EmbarkStudios/tame-gcs/compare/0.6.1...0.7.0
[0.6.1]: https://github.com/EmbarkStudios/tame-gcs/compare/0.6.0...0.6.1
[0.6.0]: https://github.com/EmbarkStudios/tame-gcs/compare/0.5.2...0.6.0
[0.5.2]: https://github.com/EmbarkStudios/tame-gcs/compare/0.5.1...0.5.2
[0.5.1]: https://github.com/EmbarkStudios/tame-gcs/compare/0.5.0...0.5.1
[0.5.0]: https://github.com/EmbarkStudios/tame-gcs/compare/0.4.1...0.5.0
[0.4.1]: https://github.com/EmbarkStudios/tame-gcs/compare/0.4.0...0.4.1
[0.4.0]: https://github.com/EmbarkStudios/tame-gcs/compare/0.3.4...0.4.0
[0.3.4]: https://github.com/EmbarkStudios/tame-gcs/compare/0.3.3...0.3.4
[0.3.3]: https://github.com/EmbarkStudios/tame-gcs/compare/0.3.2...0.3.3
[0.3.2]: https://github.com/EmbarkStudios/tame-gcs/compare/0.3.1...0.3.2
[0.3.1]: https://github.com/EmbarkStudios/tame-gcs/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/EmbarkStudios/tame-gcs/compare/0.2.0...0.3.0
[0.2.0]: https://github.com/EmbarkStudios/tame-gcs/compare/0.1.0...0.2.0
[0.1.0]: https://github.com/EmbarkStudios/tame-gcs/releases/tag/0.1.0
