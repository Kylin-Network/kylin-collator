# Kylin Collator

This repository is set up to use [Docker](https://www.docker.com/). Composing up with Docker will automatically launch a local network containing multiple validators (polkadot nodes) and collators (kylin nodes) as well as the full user interface on port 3001. However, you can build your network from source if you prefer.

## Local Development

Follow these steps to prepare a local development environment :hammer_and_wrench:

### Setup
[Rust development environment](https://substrate.dev/docs/en/knowledgebase/getting-started).

## Build

Checkout code
```bash
git clone --recursive https://github.com/Kylin-Network/kylin-collator.git

cd kylin-collator
git submodule update --recursive --remote
```
### Docker

Build debug version

```bash
docker-compose -f scripts/docker-compose.yml up -d
```
- The `scripts` directory contains multiple compose files which can be used to launch various network configurations.  

Ensure docker containers are running
```bash
docker ps
``````
- These container names should have a status of `up`:
    - launch
    - frontend  

You can access your network's secure user interface at port `3001`.

### From source
#### Build kylin node

```bash
cargo build
```

Build release version

```bash
cargo build --release
```

#### Build polkadot node
```bash
git clone https://github.com/paritytech/polkadot.git

cd polkadot
cargo build --release
```

#### Create local chain spec
```bash
# Generate rococo-local spec file
./target/release/polkadot build-spec --chain rococo-local --raw --disable-default-bootnode > rococo-local.json
```

#### Start relay chain validators
```bash
# Start Alice
./target/release/polkadot --alice --validator --base-path cumulus_relay/alice --chain rococo-local.json --port 30333 --ws-port 9944

# Start Bob
./target/release/polkadot --bob --validator --base-path cumulus_relay/bob --chain rococo-local.json --port 30334 --ws-port 9943
```

#### Create genesis & WASM files
```bash
cd kylin-collator

# Genesis
./target/release/kylin-collator export-genesis-state --parachain-id 2000 > para-2000-genesis-local

# WASM
./target/release/kylin-collator export-genesis-wasm > para-wasm-local
```

#### Start a collator node
```bash
# Customize the --chain flag for the path to your 'rococo-local.json' file
./target/release/kylin-collator --alice --collator --force-authoring --parachain-id 2000 --base-path cumulus_relay/kylin-collator --port 40333 --ws-port 8844 -- --execution wasm --chain <path to 'rococo-local.json' file> --port 30343 --ws-port 9942
```
- You should see your collator node running and peering with the already running relay chain nodes.    
- Your parachain will not begin authoring blocks until you have registered it on the relay chain.

## Interact
#### Polkadot.js
1. Connect to polkadot.js using a secure frontend connection like [apps](https://github.com/Kylin-Network/apps) or our pre-built ```frontend``` Docker container.
2. Fill in config in `Settings` -> `Developer`
```js
{
  "Address": "MultiAddress",
  "LookupSource": "MultiAddress",
  "DataInfo": {
    "url": "Text",
    "data": "Text"
  },
  "PriceFeedingData": {
    "para_id": "ParaId",
    "currencies": "Text",
    "requested_block_number": "BlockNumber",
    "processed_block_number": "Option<BlockNumber>",
    "requested_timestamp": "u128",
    "processed_timestamp": "Option<u128>",
    "payload": "Text"
  }
}
```

#### Register the parachain
1. Switch to custom endpoint 9944 for sudo access
2. Select `Developer` -> `Sudo`
3. Submit the following transaction to register your parachain
![example of registering a parachain](./doc/imgs/registerParachain.png)

#### Validate the parachain is registered
1. Verify parathread is registered
    - On custom endpoint 9944, select `Network` -> `Parachains`
    - On the parathreads tab you should see your paraid with a lifecycle status of `Onboarding`
    - After onboarding is complete you will see your parachain registered on the Overview tab
2. Verify parachain is producing blocks
    - Navigate to the collator node's custom endpoint 9942
    - Select `Network` -> `Explorer`
    - New blocks are being created if the value of `best` and `finalized` are incrementing higher

#### Submit data request
1. Ensure you are on a collator's custom endpoint, either 9942 or 9943
2. Submit a price request using the `requestPriceFeed` extrinsic 
![example of submitting a price request](./doc/imgs/requestPriceFeed.png)
