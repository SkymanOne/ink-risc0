
## Run node
In root, run `cargo build --release`. Run this node.
## Build and upload contract
In `./flipper`, run `cargo contracts build`. Upload contract to contracts ui

## Prove and submit proof to local contract
Go to `./provers/multiply`. Run `cargo run -- --contract-address {your contract address}`