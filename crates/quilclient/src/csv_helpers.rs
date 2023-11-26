use quilibrium::node::{ClockFrame, NetworkInfo, PeerId, PeerInfo};

pub fn clock_frames_to_rows(
    clock_frames: impl IntoIterator<Item = ClockFrame>,
) -> Vec<ClockFrameRow> {
    clock_frames
        .into_iter()
        .map(|clock_frame| {
            let ClockFrame {
                filter,
                frame_number,
                timestamp,
                difficulty,
            } = clock_frame;

            ClockFrameRow {
                filter: filter.to_string(),
                frame_number,
                timestamp: timestamp.to_string(),
                difficulty,
            }
        })
        .collect()
}

/// Flatten peer info into a list of peer info rows where each row has a single multiaddr.
pub fn peer_infos_to_rows(peer_infos: impl IntoIterator<Item = PeerInfo>) -> Vec<PeerInfoRow> {
    peer_infos
        .into_iter()
        .flat_map(|peer_info| {
            let PeerInfo {
                peer_id,
                multiaddrs,
                max_frame,
                timestamp,
                version,
                ..
            } = peer_info;
            multiaddrs.into_iter().map(move |multiaddr| PeerInfoRow {
                peer_id,
                multiaddr,
                max_frame,
                version: display_version(&version),
                timestamp: timestamp.to_string(),
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

/// Clock frame where the filter and the timestamp are human readable strings.
/// Useful for CSV output.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ClockFrameRow {
    pub filter: String,
    pub frame_number: u64,
    pub timestamp: String,
    pub difficulty: u32,
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
    pub version: String,
    pub timestamp: String,
}

fn display_version(version: &[u8; 3]) -> String {
    format!("{}.{}.{}", version[0], version[1], version[2])
}
