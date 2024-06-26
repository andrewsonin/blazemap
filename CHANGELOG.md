# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [0.5.0] - 2024-04-27
### Added
- `key` method for types generated by the `define_key_wrapper_bounded!` macro.
- Generate documentation for type methods created by type-generating macros.

## [0.4.0] - 2024-04-24
### Added
- Hide public re-exports of external crates used.
### Removed
- Re-exports of key-type generating macros in `prelude`.

## [0.3.0] - 2024-04-22
### Added
- `BlazeMap` collection
- Key-type generating macros:
  - `define_key_wrapper!`
  - `define_key_wrapper_bounded!`
  - `define_plain_id!`

<!-- next-url -->
[0.5.0]: https://github.com/andrewsonin/blazemap/releases/tag/v0.5.0
[0.4.0]: https://github.com/andrewsonin/blazemap/releases/tag/v0.4.0
[0.3.0]: https://github.com/andrewsonin/blazemap/releases/tag/v0.3.0