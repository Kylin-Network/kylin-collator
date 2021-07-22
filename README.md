# Kylin Node

## Local Development

Follow these steps to prepare a local development environment :hammer_and_wrench:

### Setup
[Rust development environment](https://substrate.dev/docs/en/knowledgebase/getting-started).


### Build

Checkout code
```bash
git clone --recursive https://github.com/Kylin-Network/kylin-node.git
```

Build debug version

```bash
cd kylin-node
cargo build
```

Build release version

```bash
cd kylin-node
cargo build --release
```

### Docker

Build debug version

```bash
cd scripts
docker-compose up -d
```

Check if the parachain and relay chains are running.
1. kylin-node should be visible
2. alice, bob & charlie should be visible

```bash
docker ps
``````




### Interact
Using [Kylin Market Front End](https://github.com/Kylin-Network/kylin-market-frontend) which can be used to interact with Kylin Node.

``` bash
git clone https://github.com/Kylin-Network/kylin-market-frontend.git
cd ./kylin-market-frontend
yarn install
```


## Run

### Lauch Relay chain. clone polkadot here and generate rococo-local.json. Check [the cumulus ](https://substrate.dev/cumulus-workshop/#/en/2-relay-chain/1-launch)

### Access the frontend & Setup Sudo Access
1. visit https://<hostname>:3001
2. change endpoint to a custom endpoint replacing localhost with the IP of the machine if you're not running the instance locally.
3. Add an account with a mnemonice seed. Provide an Account and a Password
4. Under Developer tab, the Sudo option will be available


### Genesis & WASM Filess

1.
```bash
/target/release/kylin-node export-genesis-state --parachain-id 2013 > para-2013-genesis-local
/target/release/kylin-node export-genesis-state --parachain-id 2013 > para-2013-wasm-local
```


### Register the chain
1. Switch custom endpoint to 9944
2. Developer -> Sudo
3. Submit the following change
paraSudoWrapper -> `sudoScheduleParaInitializeId`
4. Use the paraid of the kylin-node based on the Docker yaml file
5. Copy the exported files (genesis & wasm) out and upload them to the genesisHead (genesis files) and validationCode (wasm file)


### Validate the parachain is registered
1. 



### Using polkadot.js
visit <https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/settings/developer>.


fill the config in Settings>>Developer.
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


### Add data request the new way (from main branch using the `request_price_feed` extrinsic)
