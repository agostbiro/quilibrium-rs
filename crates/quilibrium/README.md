# Quilibrium Client Library

## Installation

1. Install [protoc](https://grpc.io/docs/protoc-installation/) which is needed to compile the protocol buffer definitions from the [Quilibrium Ceremony Client](https://github.com/QuilibriumNetwork/ceremonyclient) repo.
1. `cargo add quilibrium`

## Example usage

```rust, no_compile
// Import the client
use quilibrium::NodeClient;

// Connect to your node
let mut client = NodeClient::new("http://1.2.3.4:5678".parse()?).await?;
// Fetch the peers from the node's peer store
let network_info = client.network_info().await?;
```

## [Docs](https://docs.rs/quilibrium/latest/quilibrium/)

## [Changelog](./CHANGELOG.md)
