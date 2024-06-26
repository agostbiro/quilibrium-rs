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

use crate::oblivious_transfer_units::ObliviousTransferUnits;
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

    /// Fetch the self-reported peer manifests that the node knows about.
    pub async fn peer_manifests(&mut self) -> Result<PeerManifestsResponse, NodeClientError> {
        let request = tonic::Request::new(node_pb::GetPeerManifestsRequest {});
        let response = self.client.get_peer_manifests(request).await?;
        response.into_inner().try_into()
    }

    /// Fetch the token info from the node.
    pub async fn token_info(&mut self) -> Result<TokenInfo, NodeClientError> {
        let request = tonic::Request::new(node_pb::GetTokenInfoRequest {});
        let response = self.client.get_token_info(request).await?;
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
    /// Invalid protocol version triple.
    #[error("Invalid protocol version triple: {0:?}")]
    InvalidVersion(Vec<u8>),
    /// Invalid u64 bytes.
    #[error("Invalid u64 bytes: {0:?}")]
    InvalidU64Bytes(Vec<u8>),
    /// Quil token conversion error.
    #[error(transparent)]
    QuilTokenError(#[from] crate::oblivious_transfer_units::QuilTokenError),
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
        Ok(Self {
            filter: value.filter.try_into()?,
            frame_number: value.frame_number,
            timestamp: convert_timestamp(value.timestamp)?,
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
    /// The libp2p peer ID of the peer.
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
    /// The libp2p peer ID of the peer.
    pub peer_id: PeerId,
    /// The [multiaddrs](https://multiformats.io/multiaddr/) of the peer.
    pub multiaddrs: Vec<multiaddr::Multiaddr>,
    /// The maximum ceremony frame number reported by the peer.
    pub max_frame: u64,
    /// The self-reported timestamp of the peer info data.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    /// The protocol version triple of the peer.
    pub version: [u8; 3],
    /// The Ed448 signature of the peer attesting to the peer info.
    pub signature: Vec<u8>,
    /// The Ed448 public key of the peer that was used to sign the message. Must match the peer id.
    pub public_key: Vec<u8>,
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
            timestamp: convert_timestamp(value.timestamp)?,
            version: value
                .version
                .try_into()
                .map_err(NodeClientError::InvalidVersion)?,
            signature: value.signature,
            public_key: value.public_key,
        })
    }
}

/// Response for get peer manifests request.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PeerManifestsResponse {
    /// The peer manifests from the node.
    pub peer_manifests: Vec<PeerManifest>,
}

impl TryFrom<node_pb::PeerManifestsResponse> for PeerManifestsResponse {
    type Error = NodeClientError;

    fn try_from(value: node_pb::PeerManifestsResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            peer_manifests: value
                .peer_manifests
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

/// A peer manifest from the node containing info about the self-reported capibilities of the node.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PeerManifest {
    /// The libp2p peer ID of the peer.
    pub peer_id: PeerId,
    /// The difficulty the self test was conducted under.
    pub difficulty: u32,
    /// The resulting local time (ms) of computing the VDF test under the difficulty.
    pub difficulty_metric: i64,
    /// The resulting local time (ms) of computing a KZG commitment for a 16 degree polynomial.
    pub commit_16_metric: i64,
    /// The resulting local time (ms) of computing a KZG commitment for a 128 degree polynomial.
    pub commit_128_metric: i64,
    /// The resulting local time (ms) of computing a KZG commitment for a 1024 degree polynomial.
    pub commit_1024_metric: i64,
    /// The resulting local time (ms) of computing a KZG commitment for a 65536 degree polynomial.
    pub commit_65536_metric: i64,
    /// The resulting local time (ms) of computing a KZG proof for a 16 degree polynomial.
    pub proof_16_metric: i64,
    /// The resulting local time (ms) of computing a KZG proof for a 128 degree polynomial.
    pub proof_128_metric: i64,
    /// The resulting local time (ms) of computing a KZG proof for a 1024 degree polynomial.
    pub proof_1024_metric: i64,
    /// The resulting local time (ms) of computing a KZG proof for a 65536 degree polynomial.
    pub proof_65536_metric: i64,
    /// The number of reported accessible logical cores.
    pub cores: u32,
    /// The total available memory in bytes.
    pub memory: u64,
    /// The total available storage in bytes.
    pub storage: u64,
    /// The highest master frame the node has.
    pub master_head_frame: u64,
}

impl TryFrom<node_pb::PeerManifest> for PeerManifest {
    type Error = NodeClientError;

    fn try_from(value: node_pb::PeerManifest) -> Result<Self, Self::Error> {
        Ok(Self {
            peer_id: PeerId::from_bytes(&value.peer_id)?,
            difficulty: value.difficulty,
            difficulty_metric: value.difficulty_metric,
            commit_16_metric: value.commit_16_metric,
            commit_128_metric: value.commit_128_metric,
            commit_1024_metric: value.commit_1024_metric,
            commit_65536_metric: value.commit_65536_metric,
            proof_16_metric: value.proof_16_metric,
            proof_128_metric: value.proof_128_metric,
            proof_1024_metric: value.proof_1024_metric,
            proof_65536_metric: value.proof_65536_metric,
            cores: value.cores,
            memory: u64_from_unpadded_be_bytes(value.memory)?,
            storage: u64_from_unpadded_be_bytes(value.storage)?,
            master_head_frame: value.master_head_frame,
        })
    }
}

/// Token supply and balance from a node.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TokenInfo {
    /// The token supply from confirmed frame data.
    pub confirmed_token_supply: ObliviousTransferUnits,
    /// The token supply, including unconfirmed frame data.
    pub unconfirmed_token_supply: ObliviousTransferUnits,
    /// The tokens owned by the node's address.
    pub owned_tokens: ObliviousTransferUnits,
}

impl TryFrom<node_pb::TokenInfoResponse> for TokenInfo {
    type Error = NodeClientError;

    fn try_from(value: node_pb::TokenInfoResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            confirmed_token_supply: value.confirmed_token_supply.as_slice().try_into()?,
            unconfirmed_token_supply: value.unconfirmed_token_supply.as_slice().try_into()?,
            owned_tokens: value.owned_tokens.as_slice().try_into()?,
        })
    }
}

fn convert_timestamp(timestamp: i64) -> Result<DateTime<Utc>, NodeClientError> {
    match Utc.timestamp_millis_opt(timestamp) {
        LocalResult::Single(timestamp) => Ok(timestamp),
        LocalResult::Ambiguous(_, _) => Err(NodeClientError::InvalidTimestamp(timestamp)),
        LocalResult::None => Err(NodeClientError::InvalidTimestamp(timestamp)),
    }
}

fn u64_from_unpadded_be_bytes(bytes: Vec<u8>) -> Result<u64, NodeClientError> {
    const U64_BYTES: usize = std::mem::size_of::<u64>();

    if bytes.len() > U64_BYTES {
        return Err(NodeClientError::InvalidU64Bytes(bytes));
    }

    let mut padded = [0; U64_BYTES];
    padded[U64_BYTES - bytes.len()..].copy_from_slice(&bytes);

    Ok(u64::from_be_bytes(padded))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_from_unpadded_be_bytes() -> Result<(), NodeClientError> {
        // u8
        assert_eq!(u64_from_unpadded_be_bytes(0_u8.to_be_bytes().to_vec())?, 0);
        assert_eq!(u64_from_unpadded_be_bytes(1_u8.to_be_bytes().to_vec())?, 1);
        assert_eq!(
            u64_from_unpadded_be_bytes((u8::MAX - 1).to_be_bytes().to_vec())?,
            (u8::MAX - 1) as u64
        );
        assert_eq!(
            u64_from_unpadded_be_bytes(u8::MAX.to_be_bytes().to_vec())?,
            u8::MAX as u64
        );
        assert_eq!(
            u64_from_unpadded_be_bytes((u8::MAX as u16 + 1).to_be_bytes().to_vec())?,
            (u8::MAX as u16 + 1) as u64
        );

        // u16
        assert_eq!(
            u64_from_unpadded_be_bytes((u16::MAX - 1).to_be_bytes().to_vec())?,
            (u16::MAX - 1) as u64
        );
        assert_eq!(
            u64_from_unpadded_be_bytes(u16::MAX.to_be_bytes().to_vec())?,
            u16::MAX as u64
        );
        assert_eq!(
            u64_from_unpadded_be_bytes((u16::MAX as u32 + 1).to_be_bytes().to_vec())?,
            (u16::MAX as u32 + 1) as u64
        );

        // u32
        assert_eq!(
            u64_from_unpadded_be_bytes((u32::MAX - 1).to_be_bytes().to_vec())?,
            (u32::MAX - 1) as u64
        );
        assert_eq!(
            u64_from_unpadded_be_bytes(u32::MAX.to_be_bytes().to_vec())?,
            u32::MAX as u64
        );
        assert_eq!(
            u64_from_unpadded_be_bytes((u32::MAX as u64 + 1).to_be_bytes().to_vec())?,
            u32::MAX as u64 + 1
        );

        // u64
        assert_eq!(
            u64_from_unpadded_be_bytes((u64::MAX - 1).to_be_bytes().to_vec())?,
            u64::MAX - 1
        );
        assert_eq!(
            u64_from_unpadded_be_bytes(u64::MAX.to_be_bytes().to_vec())?,
            u64::MAX
        );
        Ok(())
    }
}
