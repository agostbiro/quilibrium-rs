//! Quilibrium client library.
//!
//! Example usage:
//! ```rust,no_run
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Import the client
//! use quilibrium::NodeClient;
//!
//! // Connect to your node
//! let mut client = NodeClient::new("http://1.2.3.4:5678".parse()?).await?;
//! // Fetch the peers from the node's peer store
//! let network_info = client.network_info().await?;
//! # Ok(())
//! # }

mod quilibrium_pb {
    pub mod node {
        pub mod channel {
            pub mod pb {
                include!(concat!(env!("OUT_DIR"), "/quilibrium.node.channel.pb.rs"));
            }
        }
        pub mod clock {
            pub mod pb {
                include!(concat!(env!("OUT_DIR"), "/quilibrium.node.clock.pb.rs"));
            }
        }
        pub mod keys {
            pub mod pb {
                include!(concat!(env!("OUT_DIR"), "/quilibrium.node.keys.pb.rs"));
            }
        }
        #[allow(clippy::module_inception)]
        pub mod node {
            pub mod pb {
                include!(concat!(env!("OUT_DIR"), "/quilibrium.node.node.pb.rs"));
            }
        }
    }
}

pub mod csv;
mod node;

pub use node::{NodeClient, NodeClientError, PeerInfo};
