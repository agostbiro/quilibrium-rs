# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Fixed

### Changed

### Removed

## [0.2.1] - 2023-11-28

### Added

- Exposed [QuilTokenError](https://docs.rs/quilibrium/latest/quilibrium/oblivious_transfer_units/enum.QuilTokenError.html) in the public interface.
- Added `timestamp`, `version`, `public_key`, `signature` fields to [PeerInfo](https://docs.rs/quilibrium/latest/quilibrium/node/struct.PeerInfo.html).

### Changed

- Made it an error to construct
  [ObliviousTransferUnits](https://docs.rs/quilibrium/latest/quilibrium/struct.ObliviousTransferUnits.html)
  from a slice shorter than 32 bytes since the Quilibrium gRPC returns them as
  32 byte vectors and passing a shorter slice is probably an error. Trying to
  construct
  [ObliviousTransferUnits](https://docs.rs/quilibrium/latest/quilibrium/struct.ObliviousTransferUnits.html)
  from a longer slice was already an error.


## [0.2.0] - 2023-10-29

### Added

- Added [frame metadata](https://docs.rs/quilibrium/0.2.0/quilibrium/node/struct.NodeClient.html#method.frames) query support to `NodeClient`.
- Added [frame info](https://docs.rs/quilibrium/0.2.0/quilibrium/node/struct.NodeClient.html#method.frame_info) support to `NodeClient`.
- Added [token info](https://docs.rs/quilibrium/0.2.0/quilibrium/node/struct.NodeClient.html#method.token_info) support to `NodeClient`.

### Changed

- Node related types are now exposed in the `node` module (as opposed to the library root).

### Removed

- Removed the `csv` module and moved it to the [quilclient](../quilclient/README.md) crate as a private module as it was tightly coupled with that crate.

## [0.1.1] - 2023-10-14

### Fixed

- Add missing debug and clone implementations to `NodeClient` and `NetworkInfoResponse`.

## [0.1.0] - 2023-10-14

### Added

- Added `NodeClient` with [network info](https://docs.rs/quilibrium/0.1.0/quilibrium/struct.NodeClient.html#method.network_info) and [peer info](https://docs.rs/quilibrium/0.1.0/quilibrium/struct.NodeClient.html#method.peer_info) support.

[unreleased]: https://github.com/agostbiro/quilibrium-rs/compare/quilibrium-0.2.0..HEAD
[0.2.0]: https://github.com/agostbiro/quilibrium-rs/compare/quilibrium-0.1.1..quilibrium-0.2.0
[0.1.1]: https://github.com/agostbiro/quilibrium-rs/compare/quilibrium-0.1.0..quilibrium-0.1.1
[0.1.0]: https://github.com/agostbiro/quilibrium-rs/compare/quilibrium-0.1.0
