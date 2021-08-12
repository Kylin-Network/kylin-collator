# Kylin Node

This repository is set up to use [Docker](https://www.docker.com/). Launching your chain with docker will automatically start two validators and launches the full user interface on port 3001. However, you can build from source if you prefer.

## Local Development

Follow these steps to prepare a local development environment :hammer_and_wrench:

### Setup
[Rust development environment](https://substrate.dev/docs/en/knowledgebase/getting-started).


## Build

Checkout code
```bash
git clone --recursive https://github.com/Kylin-Network/kylin-node.git

cd kylin-node
git submodule update --recursive --remote
```

Build debug version

```bash
cargo build
```

Build release version

```bash
cargo build --release
```

### Docker

Build debug version

```bash
cd scripts
docker-compose up -d
```

Ensure docker containers are running.
```bash
docker ps
``````
- These container names should have a status of 'up':
    - launch
    - frontend

## Run

### Launch Relay Chain

#### Build Polkadot Node
```bash
git clone https://github.com/paritytech/polkadot.git

cd polkadot
cargo build --release
```

#### Create Local Chain Spec
```bash
# Generate rococo-local spec file
./target/release/polkadot build-spec --chain rococo-local --raw --disable-default-bootnode > rococo-local.json
```

#### Start Relay Chain Validators
```bash
# Start Alice
./target/release/polkadot --alice --validator --base-path cumulus_relay/alice --chain rococo-local.json --port 30333 --ws-port 9944

# Start Bob
./target/release/polkadot --bob --validator --base-path cumulus_relay/bob --chain rococo-local.json --port 30334 --ws-port 9943
```

#### Create Genesis & WASM Files
```bash
cd kylin-node

# Genesis
./target/release/kylin-node export-genesis-state --parachain-id 2000 > para-2000-genesis-local

# WASM
./target/release/kylin-node export-genesis-wasm > para-wasm-local
```

#### Start the Collator Node
```bash
# Customize the --chain flag for the path to your 'rococo-local.json' file
./target/release/kylin-node --alice --collator --force-authoring --parachain-id 2000 --base-path cumulus_relay/kylin-node --port 40333 --ws-port 8844 -- --execution wasm --chain <path to 'rococo-local.json' file> --port 30343 --ws-port 9942
```
If all goes well, you should see your collator node running and peering with the already running relay chain nodes.  
Your parachain will not begin authoring blocks until you have registered it on the relay chain.

### Interact
#### Polkadot.js
1. Navigate to [polkadot.js](https://polkadot.js.org/apps/#/explorer)
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
1. Switch to Alice endpoint 9944 for Sudo access
2. Select `Developer` -> `Sudo`
3. Submit the following transaction
    - `paraSudoWrapper` -> `sudoScheduleParaInitializeId`
        - paraid -> 2000
        - Upload or paste genesis and wasm files
            - genesisHead -> para-2000-genesis-local
            - validationCode -> para-wasm-local
        - parachain -> True

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
1. Ensure you are on the parachain's custom endpoint, 9942
2. Submit a price request using the `requestPriceFeed` extrinsic 
![submitting price request](./doc/imgs/requestPriceFeed.png)
    
