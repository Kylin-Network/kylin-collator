# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog] and this project adheres to
[Semantic Versioning].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: http://semver.org/spec/v2.0.0.html

## [0.1.4] - 2019-10-19

### Changed
- The separator is now a `&str` rather than a `char`.
- Oldest supported rustc version is now 1.22.0.

### Added
- A non-blanket `impl Separable for str` allocates only once, for the result,
rather than twice as the blanket `Display`-based `impl` does.
- An empty group array results in no separators.

### Fixed
- Now respects UTF-8 for the separator as well.

## [0.1.4] - 2019-10-19

### Fixed
- Now respects UTF-8.

## [0.1.3] - 2019-10-19

### Added
- `Separable::separate_with_underscores` method and
`policies::UNDERSCORE_SEPARATOR` constant.

## [0.1.2] - 2018-09-18

### Fixed
- Github URL in Cargo.toml.

## [0.1.1] - 2018-09-16

### Added
- Doc comment for `policies` module.

## [0.1.0] - 2018-09-16

Initial release.

