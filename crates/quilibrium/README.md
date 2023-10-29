# Quilibrium Client Library

## Installation

`cargo add quilibrium`

## Example usage

```rust
// Import the client
use quilibrium::NodeClient;

// Connect to your node
let mut client = NodeClient::new("http://1.2.3.4:5678".parse()?).await?;
// Fetch the peers from the node's peer store
let network_info = client.network_info().await?;
```

## [Docs](https://docs.rs/quilibrium/latest/quilibrium/)

## [Changelog](./CHANGELOG.md)
