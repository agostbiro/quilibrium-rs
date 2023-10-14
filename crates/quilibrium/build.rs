fn main() {
    tonic_build::configure()
        .build_server(false)
        .type_attribute(
            "quilibrium.node.node.pb.PeerInfoResponse",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "quilibrium.node.node.pb.PeerInfo",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile(
            &["protobufs/ceremonyclient/node/protobufs/node.proto"],
            &["protobufs/ceremonyclient/node/protobufs"],
        )
        .unwrap();
}
