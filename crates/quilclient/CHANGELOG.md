# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Fixed

### Changed

### Removed

## [0.2.0] - 2023-10-29

### Added

- Added CLI support for fetching frame metadata and printing it in CSV format to stdout.
- Added CLI support to download a clock frame in Protocol Buffer format.
- Added CLI support to fetch the token balance of the node and print it to stdout in QUIL units as an integer.
- Added CLI support to fetch the confirmed token supply and print it to stdout in QUIL units as an integer. 

## [0.1.0] - 2023-10-14

### Added

- Added CLI support to fetch the peers from the node's peer store and print them to stdout as CSV.
- Added CLI support to fetch the broadcasted sync info that gets replicated through the network mesh and print it to stdout as CSV. 

[unreleased]: https://github.com/agostbiro/quilibrium-rs/compare/quilclient-0.2.0..HEAD
[0.2.0]: https://github.com/agostbiro/quilibrium-rs/compare/quilclient-0.1.0..quilclient-0.2.0
[0.1.0]: https://github.com/agostbiro/quilibrium-rs/compare/quilclient-0.1.0
