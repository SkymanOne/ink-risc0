# Factors Prover
Proves hello world/factors example

## Getting chain metadata
The chain metadata changes with any alterations in the runtime. The chain metadata in `metadata.scale` needs to be updated each time. To do this:

Start local node
run
```shell
subxt metadata -f bytes > metadata.scale
```

## Image ID
The Substrate pallet knows and trusts the image id of this guest. After each change, the image id will need to be updated here `pallets/template/src/common.rs`, and the substrate runtime also rebuilt(run `cargo build --release` in project root). 
