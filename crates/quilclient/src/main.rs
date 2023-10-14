use anyhow::Result;
use clap::error::ErrorKind;
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use serde::Serialize;
use tonic::transport::Uri;

use quilibrium::csv::network_infos_to_rows;
use quilibrium::{csv::peer_infos_to_rows, NodeClient, PeerInfo};

/// Quilibrium CLI client.
#[derive(Debug, Parser)]
#[clap(name = "quilclient", version)]
pub struct QuilClientArgs {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Command,
}

static QUILCLIENT_NODE_URI: &str = "QUILCLIENT_NODE_URI";

#[derive(Debug, Args)]
struct GlobalOpts {
    /// The gRPC URI of the Quilibrium node, e.g. <http://1.2.3.4:5678>.
    /// See the Ceremony Client readme for more:
    /// <https://github.com/quilibriumnetwork/ceremonyclient#experimental--grpcrest-support>
    #[clap(long, short, global = true, env = QUILCLIENT_NODE_URI)]
    node_uri: Option<Uri>,
}

/// Quilibrium CLI client commands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Fetch the peers from the node's peer store and print them to stdout as CSV.
    NetworkInfo,
    /// Fetch the broadcasted sync info that gets replicated through the network mesh and print it to stdout as CSV.
    PeerInfo {
        #[clap(value_enum, default_value_t=PeerType::Cooperative)]
        peer_type: PeerType,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum PeerType {
    Cooperative,
    Uncooperative,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = QuilClientArgs::parse();

    // Hack to work around not being able to make global args required:
    // https://github.com/clap-rs/clap/issues/1546
    let node_uri = args.global_opts.node_uri.unwrap_or_else(|| {
        let mut cmd = QuilClientArgs::command();
        cmd.error(
            ErrorKind::MissingRequiredArgument,
            format!("The --node_uri argument or the {QUILCLIENT_NODE_URI} must be set"),
        )
        .exit();
    });

    let mut client = NodeClient::new(node_uri).await?;

    match args.command {
        Command::NetworkInfo => {
            let network_info = client.network_info().await?;
            write_csv_to_stdout(network_infos_to_rows(network_info.network_info)).await?;
        }
        Command::PeerInfo { peer_type } => {
            let peer_info = client.peer_info().await?;
            match peer_type {
                PeerType::Cooperative => write_peer_infos(peer_info.peers).await?,
                PeerType::Uncooperative => write_peer_infos(peer_info.uncooperative_peers).await?,
            };
        }
    }

    Ok(())
}

async fn write_peer_infos(peer_infos: impl IntoIterator<Item = PeerInfo>) -> Result<()> {
    write_csv_to_stdout(peer_infos_to_rows(peer_infos)).await
}

async fn write_csv_to_stdout(
    rows: impl IntoIterator<Item = impl Serialize> + Send + 'static,
) -> Result<()> {
    tokio::task::spawn_blocking::<_, Result<()>>(move || {
        let mut wtr = csv::Writer::from_writer(std::io::stdout());
        for row in rows {
            wtr.serialize(row)?;
        }
        wtr.flush()?;
        Ok(())
    })
    .await??;
    Ok(())
}
