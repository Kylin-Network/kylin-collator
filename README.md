# Kylin Node

This repository is set up to use `docker`. Launching your chain with docker will automatically start two validators and launches the full user interface on port 3001. However, you can build from source if you prefer.

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

#### Create Chain Spec
```bash
# Generate rococo-local.json spec file
target/release/polkadot build-spec --chain rococo-local --disable-default-bootnode > rococo-custom-plain.json

# Generate final 'raw' spec file
target/release/polkadot build-spec --chain rococo-custom-plain.json --raw --disable-default-bootnode > rococo-custom.json
```

#### Start Relay Chain Validators
```bash
# Start Alice
target/release/polkadot --alice --validator --base-path cumulus_relay/alice --chain rococo-custom.json --port 30333 --ws-port 9944

# Start Bob
target/release/polkadot --bob --validator --base-path cumulus_relay/bob --chain rococo-custom.json --port 30334 --ws-port 9945
```

#### Create Genesis & WASM Files
```bash
# Genesis file
target/debug/kylin-node export-genesis-state --parachain-id 2000 > para-genesis-local

# WASM file
target/release/kylin-node export-genesis-wasm > para-wasm-local
```

### Interact
#### Polkadot.js
1. Navigate to polkadot.js
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
1. Switch to Alice endpoint `9944` for Sudo access
2. Select `Developer` -> `Sudo`
3. Submit the following transaction
    - `paraSudoWrapper` -> `sudoScheduleParaInitializeId`
        - `paraid` -> use value passed as `--parachain-id` for kylin-node in yml file
        - Upload or paste Genesis and WASM data from exported files
            - `genesisHead` -> para-genesis-local
            - `validationCode` -> para-wasm-local
        - `parachain` -> True

#### Validate the parachain is registered
1. Verify parathread is registered
    - On custom endpoint 9944, select `Network` -> `Parachains`
    - On the `parathreads` tab you should see your `paraid` with a `lifecycle` status of 'Onboarding'
    - After onboarding is complete you will see your parachain registered on the `Overview` tab
2. Verify parachain is producing blocks
    - Navigate to custom endpoint 9942
    - Select `Network` -> `Explorer`
    - New blocks are being created if the value of `best` and `finalized` are incrementing higher


#### Submit data requests
1. Navigate to kylin-node custom endpoint `9942`
2. Submit a price request
    - ![submitting price request](./doc/imgs/requestPriceFeed.png)
