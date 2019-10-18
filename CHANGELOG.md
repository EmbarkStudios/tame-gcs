# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Added `content_disposition` to `objects::Metadata`

## [0.5.0] - 2019-10-10
### Changed
- Update dependencies
- Replace use of `failure` with `thiserror` for the library
- Replace use of `failure` with `anyhow` in the examples

## [0.4.0] - 2019-08-02
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

[Unreleased]: https://github.com/EmbarkStudios/tame-gcs/compare/0.5.0...HEAD
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
