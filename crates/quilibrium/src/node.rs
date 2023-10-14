use libp2p_identity::PeerId;
use tonic::transport::Uri;

use crate::quilibrium_pb::node::node::pb::{
    self as node_pb, node_service_client::NodeServiceClient,
};

/// gRPC client for a Quilibrium node.
pub struct NodeClient {
    client: NodeServiceClient<tonic::transport::Channel>,
}

impl NodeClient {
    /// Create a new node client. The URI should be the address of the [node's gRPC
    /// service.](https://github.com/quilibriumnetwork/ceremonyclient#experimental--grpcrest-support)
    pub async fn new(uri: Uri) -> Result<Self, NodeClientError> {
        let client = NodeServiceClient::connect(uri).await?;
        Ok(Self { client })
    }

    /// Fetch the peers from the node's peer store.
    pub async fn network_info(&mut self) -> Result<NetworkInfoResponse, NodeClientError> {
        let request = tonic::Request::new(node_pb::GetNetworkInfoRequest {});
        let response = self.client.get_network_info(request).await?;
        response.into_inner().try_into()
    }

    /// Fetch the broadcasted sync info that gets replicated through the network mesh.
    pub async fn peer_info(&mut self) -> Result<PeerInfoResponse, NodeClientError> {
        let request = tonic::Request::new(node_pb::GetPeerInfoRequest {});
        let response = self.client.get_peer_info(request).await?;
        response.into_inner().try_into()
    }
}

/// Errors that can occur when interacting with a node.
#[derive(Debug, thiserror::Error)]
pub enum NodeClientError {
    #[error(transparent)]
    InvalidMultiaddr(#[from] multiaddr::Error),
    #[error(transparent)]
    InvalidPeerId(#[from] libp2p_identity::ParseError),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
}

/// A network info response from a node.
pub struct NetworkInfoResponse {
    pub network_info: Vec<NetworkInfo>,
}

impl TryFrom<node_pb::NetworkInfoResponse> for NetworkInfoResponse {
    type Error = NodeClientError;

    fn try_from(value: node_pb::NetworkInfoResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            network_info: value
                .network_info
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

/// Info about a peer from the node's peer store.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfo {
    pub peer_id: PeerId,
    pub multiaddrs: Vec<multiaddr::Multiaddr>,
    pub peer_score: f64,
}

impl TryFrom<node_pb::NetworkInfo> for NetworkInfo {
    type Error = NodeClientError;

    fn try_from(value: node_pb::NetworkInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            peer_id: PeerId::from_bytes(&value.peer_id)?,
            multiaddrs: value
                .multiaddrs
                .iter()
                .map(|m| m.parse())
                .collect::<Result<_, _>>()?,
            peer_score: value.peer_score,
        })
    }
}

/// Info about a peer the node knows about.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PeerInfoResponse {
    /// The cooperative peers the node knows about.
    pub peers: Vec<PeerInfo>,
    /// The uncooperative peers the node knows about.
    pub uncooperative_peers: Vec<PeerInfo>,
}

impl TryFrom<node_pb::PeerInfoResponse> for PeerInfoResponse {
    type Error = NodeClientError;

    fn try_from(value: node_pb::PeerInfoResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            peers: value
                .peer_info
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            uncooperative_peers: value
                .uncooperative_peer_info
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

/// Information about a peer the node knows about from the broadcasted sync info that gets
/// replicated through the network mesh.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub multiaddrs: Vec<multiaddr::Multiaddr>,
    pub max_frame: u64,
}

impl TryFrom<node_pb::PeerInfo> for PeerInfo {
    type Error = NodeClientError;

    fn try_from(value: node_pb::PeerInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            peer_id: PeerId::from_bytes(&value.peer_id)?,
            multiaddrs: value
                .multiaddrs
                .iter()
                .map(|m| m.parse())
                .collect::<Result<_, _>>()?,
            max_frame: value.max_frame,
        })
    }
}
