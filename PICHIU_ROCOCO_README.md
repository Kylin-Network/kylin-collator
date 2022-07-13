# Kylin Collator on Pichiu/Rococo



These are instructions for starting a collator on Pichiu/Rococo test net.

### Setup

```bash
git clone --recursive https://github.com/Kylin-Network/kylin-collator.git
```
```
cd kylin-collator
```
```
git submodule init
git submodule update 
git checkout 99f1199298128eecb5eb55b09d11c5c12283bd5f
```

Setup your [Rust development environment](https://substrate.dev/docs/en/knowledgebase/getting-started). 
  
```
rustup default nightly
rustup update
rustup target add wasm32-unknown-unknown --toolchain nightly
cargo build --release
```  
 
#### Start a collator on pichiu/rococo

###bare metal

```bash
target/release/kylin-collator --collator --bootnodes /ip4/35.78.250.13/tcp/40333/p2p/12D3KooWQ3stLjQa4R1Rrccw1s9ViZHna37iuosaAcS2bmzUn9oe  --unsafe-ws-external  --name pichiu-collator-<your id> --force-authoring --parachain-id 2102 --chain ./pichiu-rococo-parachain-2102.json --port 40333 --ws-port 8844 --rpc-cors all --log parachain:debug  -- --execution wasm --chain ./rococo.json --port 30343 --ws-port 9977 
```

- You should see your collator running and peering with the already running relay chain validators.

### docker
docker-compose up
