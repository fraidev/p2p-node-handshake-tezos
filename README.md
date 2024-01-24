# p2p-node-handshake-tezos


This is a [coding chanllenge](https://github.com/eqlabs/recruitment-exercises/blob/master/node-handshake.md) for `eqlabs`.


# How to run

To find a peer from the bootstrap peers and try a handshake, just run in your terminal:

```bash
cargo run
```

<img width="2438" alt="image" src="https://github.com/fraidev/p2p-node-handshake-tezos/assets/25258368/56e8fa19-0d64-4623-b263-0c0ccda2c85f">

# How to run with a local Tezos node

In order to run with a local node, you need to run a local node first. You can follow the instructions [here](https://tezos.gitlab.io/introduction/howtoget.html#build-from-sources) to build a local node from source.

I recommend to import a snapshot to not wait the node to sync. You can find the snapshots [here](https://mainnet.tezos.marigold.dev/) (by the way, I created this snapshot site 😊)


# Customization

You can put args to run the handshake with a local node and a custom identity file or a different tezos network.
After you have a local node running, you can test the handshake with the local node by running:

```bash
cargo run {local_node_address} {identity_file_path} {tezos_network}
```

Example to run with a local node, a custom identity file and using the Ghostnet (old Ithacanet) network:

```bash
cargo run 127.0.0.1:9732 identity.json TEZOS_ITHACANET_2022-01-25T15:00:00Z
```

