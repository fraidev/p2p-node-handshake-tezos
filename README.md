# p2p-node-handshake-tezos

This is a simple Rust program to test the handshake between a Tezos nodes.

# How to run

To find a peer from the bootstrap peers and try a handshake, just run in your terminal:

```bash
cargo run
```

<img width="2438" alt="image" src="https://github.com/fraidev/p2p-node-handshake-tezos/assets/25258368/56e8fa19-0d64-4623-b263-0c0ccda2c85f">

# How to run with a local Tezos node

To run with a local node, you need to run a local node first. You can follow the instructions [here](https://tezos.gitlab.io/introduction/howtoget.html#build-from-sources) to build a local node from source.

I recommend importing a snapshot to not wait the node to sync. You can find the snapshots [here](https://mainnet.tezos.marigold.dev/) (by the way, I created this snapshot site 😊)

# Customization

You can put arguments to run the handshake with a local node and a custom identity file or a different Tezos network.
After you have a local node running, you can test the handshake with the local node by running:

```bash
cargo run {local_node_address} {identity_file_path} {tezos_network}
```

Example to run with a local node, a custom identity file and using the Ghostnet (old Ithacanet) network:

```bash
cargo run 127.0.0.1:9732 identity.json TEZOS_ITHACANET_2022-01-25T15:00:00Z
```

With the octez node installed, and snapshot downloaded, you can run the following this script to initialize the node, import the snapshot, run the node and test the handshake

```bash
#!/usr/bin/env sh
OCTEZ_NODE_DIR="tezos-node/"
NETWORK="mainnet"
SNAPSHOT_FILE="TEZOS_MAINNET-BMXVtPg43aUUUNcyGX6fKKUQaUS75eajUGtvKL4AbzN9r2wDm4z-4971506.full"
OCTEZ_NODE="$HOME/tezos/octez-node"
HISTORY_MODE="full"

# Init node
$OCTEZ_NODE config init --data-dir "${OCTEZ_NODE_DIR}" --network "${NETWORK}" --history-mode="${HISTORY_MODE}" --net-addr="[::]:9732" --rpc-addr="127.0.0.1:8732"

# Import snapshot
$OCTEZ_NODE snapshot import ${SNAPSHOT_FILE} --data-dir ${OCTEZ_NODE_DIR} --config-file ${ONODE_DIR}/config.json --no-check

# Run node
$OCTEZ_NODE run --data-dir "${OCTEZ_NODE_DIR}" --network "${NETWORK}" --rpc-addr 0.0.0.0:8732

# Run handshake
cargo run 127.0.0.1:9732
```
