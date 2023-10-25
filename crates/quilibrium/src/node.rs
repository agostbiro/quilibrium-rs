//! gRPC client for a Quilibrium node.
//!
//! Example usage:
//! ```rust,no_run
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Import the client
//! use quilibrium::node::NodeClient;
//!
//! // Connect to your node
//! let mut client = NodeClient::new("http://1.2.3.4:5678".parse()?).await?;
//! // Fetch the peers from the node's peer store
//! let network_info = client.network_info().await?;
//! # Ok(())
//! # }

use chrono::{DateTime, LocalResult, TimeZone, Utc};
use lazy_static::lazy_static;
pub use libp2p_identity::PeerId;
use std::fmt::Display;
use tonic::transport::Uri;

use crate::quilibrium_pb::node::node::pb::GetFrameInfoRequest;
use crate::quilibrium_pb::node::{
    clock::pb::{self as clock_pb},
    node::pb::{self as node_pb, node_service_client::NodeServiceClient},
};

/// gRPC client for a Quilibrium node.
#[derive(Debug, Clone)]
pub struct NodeClient {
    client: NodeServiceClient<tonic::transport::Channel>,
}

const MAX_DECODING_MESSAGE_SIZE_BYTES: usize = 25 * 1024 * 1024;

impl NodeClient {
    /// Create a new node client. The URI should be the address of the [node's gRPC
    /// service.](https://github.com/quilibriumnetwork/ceremonyclient#experimental--grpcrest-support)
    pub async fn new(uri: Uri) -> Result<Self, NodeClientError> {
        let client = NodeServiceClient::connect(uri)
            .await?
            .max_decoding_message_size(MAX_DECODING_MESSAGE_SIZE_BYTES);
        Ok(Self { client })
    }

    /// Get frame metadata for a frame filter.
    pub async fn frames(
        &mut self,
        options: FramesOptions,
    ) -> Result<FramesResponse, NodeClientError> {
        let request = tonic::Request::new(options.into());
        let response = self.client.get_frames(request).await?;
        response.into_inner().try_into()
    }

    /// Get a frame by frame filter and frame number.
    pub async fn frame_info(
        &mut self,
        filter: FrameFilter,
        frame_number: u64,
    ) -> Result<Option<clock_pb::ClockFrame>, NodeClientError> {
        let request = tonic::Request::new(GetFrameInfoRequest {
            filter: filter.into(),
            frame_number,
            selector: vec![],
        });
        let response = self.client.get_frame_info(request).await?;
        Ok(response.into_inner().clock_frame)
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
    /// Invalid frame filter.
    #[error("Invalid frame filter")]
    InvalidFrameFilter,
    /// The [multiaddr](https://multiformats.io/multiaddr/) is invalid.
    #[error(transparent)]
    InvalidMultiaddr(#[from] multiaddr::Error),
    #[error(transparent)]
    /// The libp2p peer ID is invalid.
    InvalidPeerId(#[from] libp2p_identity::ParseError),
    /// Invalid Unix timestamp.
    #[error("Invalid Unix timestamp: {0}")]
    InvalidTimestamp(i64),
    /// gRPC call error.
    #[error(transparent)]
    Status(#[from] tonic::Status),
    /// HTTP client error.
    #[error(transparent)]
    Transport(#[from] tonic::transport::Error),
}

/// Options for a get frames request.
pub struct FramesOptions {
    /// The frame filter.
    pub filter: FrameFilter,
    /// The frame number to start from, inclusive.
    pub from_frame_number: u64,
    /// The frame number to end at, exclusive.
    pub to_frame_number: u64,
    /// Whether to include candidate frames.
    pub include_candidates: bool,
}

impl Default for FramesOptions {
    fn default() -> Self {
        Self {
            filter: FrameFilter::MasterClock,
            from_frame_number: 0,
            to_frame_number: 1,
            include_candidates: false,
        }
    }
}

impl FramesOptions {
    /// Create a new frames options builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the frame filter.
    pub fn filter(mut self, filter: FrameFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Set the frame number to start from.
    pub fn from_frame_number(mut self, from_frame_number: u64) -> Self {
        self.from_frame_number = from_frame_number;
        self
    }

    /// Set the frame number to end at.
    pub fn to_frame_number(mut self, to_frame_number: u64) -> Self {
        self.to_frame_number = to_frame_number;
        self
    }

    /// Set whether to include candidate frames.
    pub fn include_candidates(mut self, include_candidates: bool) -> Self {
        self.include_candidates = include_candidates;
        self
    }
}

/// A get frames response from a node.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FramesResponse {
    /// The clock frames in the response.
    pub truncated_clock_frames: Vec<ClockFrame>,
}

impl TryFrom<node_pb::FramesResponse> for FramesResponse {
    type Error = NodeClientError;

    fn try_from(value: node_pb::FramesResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            truncated_clock_frames: value
                .truncated_clock_frames
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<FramesOptions> for node_pb::GetFramesRequest {
    fn from(value: FramesOptions) -> Self {
        Self {
            filter: value.filter.into(),
            from_frame_number: value.from_frame_number,
            to_frame_number: value.to_frame_number,
            include_candidates: value.include_candidates,
        }
    }
}

/// Represents a clock frame for a given filter. Clock frames are the primary
/// sequencing mechanism upon which the network derives consensus. As the master
/// pulse clock, this provides deterministic but random leader election. At the
/// data pulse clock level, this provides the same, within a quorum for data
/// sequencers.
/// Docs from: https://github.com/QuilibriumNetwork/ceremonyclient/blob/20aae290cb2f67b4557bd3aa245193f4a4992583/node/protobufs/clock.proto
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClockFrame {
    /// The filter is used as a domain separator for the input.
    pub filter: FrameFilter,
    /// A strictly monotonically-increasing frame number. Used for culling old
    /// frames past a configurable cutoff point.
    pub frame_number: u64,
    /// The self-reported timestamp from the proof publisher, encoded as an int64
    /// of the Unix epoch in milliseconds.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    /// The difficulty level used for the frame.
    pub difficulty: u32,
}

impl TryFrom<clock_pb::ClockFrame> for ClockFrame {
    type Error = NodeClientError;

    fn try_from(value: clock_pb::ClockFrame) -> Result<Self, Self::Error> {
        let timestamp = match Utc.timestamp_millis_opt(value.timestamp) {
            LocalResult::Single(timestamp) => timestamp,
            LocalResult::Ambiguous(_, _) => {
                Err(NodeClientError::InvalidTimestamp(value.timestamp))?
            }
            LocalResult::None => Err(NodeClientError::InvalidTimestamp(value.timestamp))?,
        };
        Ok(Self {
            filter: value.filter.try_into()?,
            frame_number: value.frame_number,
            timestamp,
            difficulty: value.difficulty,
        })
    }
}

const FRAME_FILTER_BYTES: usize = 32;

lazy_static! {
    static ref MASTER_CLOCK_FRAME_FILTER: [u8; FRAME_FILTER_BYTES] =
        hex::decode("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
            .expect("valid hex")
            .try_into()
            .expect("FRAME_FILTER_BYTES long");
    static ref CEREMONY_APPLICATION_FRAME_FILTER: [u8; FRAME_FILTER_BYTES] =
        hex::decode("34001BE7432C2E6669ADA0279788682AB9F62671B1B538AB99504694D981CBD3")
            .expect("valid hex")
            .try_into()
            .expect("FRAME_FILTER_BYTES long");
}

/// A frame filter for Quilibrium clock frames.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FrameFilter {
    /// The ceremony application frame filter: "34001BE7432C2E6669ADA0279788682AB9F62671B1B538AB99504694D981CBD3"
    CeremonyApplication,
    /// The master clock frame filter: "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"
    MasterClock,
    /// An unknown frame filter.
    Unknown([u8; FRAME_FILTER_BYTES]),
}

impl Display for FrameFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            FrameFilter::CeremonyApplication => write!(f, "ceremony-application"),
            FrameFilter::MasterClock => write!(f, "master-clock"),
            FrameFilter::Unknown(filter) => write!(f, "unknown-{}", hex::encode(filter)),
        }
    }
}

impl TryFrom<Vec<u8>> for FrameFilter {
    type Error = NodeClientError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value == MASTER_CLOCK_FRAME_FILTER.as_slice() {
            Ok(Self::MasterClock)
        } else if value == CEREMONY_APPLICATION_FRAME_FILTER.as_slice() {
            Ok(Self::CeremonyApplication)
        } else if value.len() == FRAME_FILTER_BYTES {
            Ok(Self::Unknown(value.try_into().expect("checked length")))
        } else {
            Err(NodeClientError::InvalidFrameFilter)
        }
    }
}

impl From<FrameFilter> for Vec<u8> {
    fn from(value: FrameFilter) -> Self {
        match value {
            FrameFilter::CeremonyApplication => CEREMONY_APPLICATION_FRAME_FILTER.to_vec(),
            FrameFilter::MasterClock => MASTER_CLOCK_FRAME_FILTER.to_vec(),
            FrameFilter::Unknown(value) => value.to_vec(),
        }
    }
}

/// A network info response from a node.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfoResponse {
    /// The network info data.
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
    /// The lip2p peer ID of the peer.
    pub peer_id: PeerId,
    /// The [multiaddrs](https://multiformats.io/multiaddr/) of the peer.
    pub multiaddrs: Vec<multiaddr::Multiaddr>,
    /// The peer score by the node.
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
    /// The lip2p peer ID of the peer.
    pub peer_id: PeerId,
    /// The [multiaddrs](https://multiformats.io/multiaddr/) of the peer.
    pub multiaddrs: Vec<multiaddr::Multiaddr>,
    /// The maximum ceremony frame number reported by the peer.
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
