use crate::node::{NetworkInfo, PeerInfo};

use libp2p_identity::PeerId;

/// Flatten peer info into a list of peer info rows where each row has a single multiaddr.
pub fn peer_infos_to_rows(peer_infos: impl IntoIterator<Item = PeerInfo>) -> Vec<PeerInfoRow> {
    peer_infos
        .into_iter()
        .flat_map(|peer_info| {
            let PeerInfo {
                peer_id,
                multiaddrs,
                max_frame,
            } = peer_info;
            multiaddrs.into_iter().map(move |multiaddr| PeerInfoRow {
                peer_id,
                multiaddr,
                max_frame,
            })
        })
        .collect()
}

/// Flatten network info into a list of network info rows where each row has a single multiaddr.
pub fn network_infos_to_rows(
    network_infos: impl IntoIterator<Item = NetworkInfo>,
) -> Vec<NetworkInfoRow> {
    network_infos
        .into_iter()
        .flat_map(|network_info| {
            let NetworkInfo {
                peer_id,
                multiaddrs,
                peer_score,
            } = network_info;
            multiaddrs.into_iter().map(move |multiaddr| NetworkInfoRow {
                peer_id,
                multiaddr,
                peer_score,
            })
        })
        .collect()
}

/// Network info where instead of a list of multiaddrs, we have a single multiaddr.
/// Useful for CSV output.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NetworkInfoRow {
    pub peer_id: PeerId,
    pub multiaddr: multiaddr::Multiaddr,
    pub peer_score: f64,
}

/// Peer info where instead of a list of multiaddrs, we have a single multiaddr.
/// Useful for CSV output.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PeerInfoRow {
    pub peer_id: PeerId,
    pub multiaddr: multiaddr::Multiaddr,
    pub max_frame: u64,
}
