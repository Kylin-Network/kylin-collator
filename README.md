# Kylin Collator
**This README is out of date, stand by for new documentation**
Using this repository, you can choose to configure and launch your network in multiple ways. Options include:
1) Launch a local development network including a relaychain and two Kylin parachains
2) Launch a collator on a live test network
3) Build from source and configure a custom network

We recommended launching with [Docker](https://www.docker.com/). The default compose file automatically launches a local network containing multiple relaychain validators (polkadot nodes) and collators (kylin collators) as well as the full user interface on port 3001. Additionally, docker can be used to launch a collator to supported testnets.

You can also build your network from source if you prefer, which is discussed step-by-step in this document as well.

## 1) Using Docker

Follow these steps to prepare a local development environment :hammer_and_wrench:

### Setup

1) You will need to [install docker](https://www.docker.com/products/docker-desktop) to launch our pre-built containers
2) Checkout the repository
```bash
git clone --recursive https://github.com/Kylin-Network/kylin-collator.git

cd kylin-collator
git submodule update --recursive --remote
```

### Run

Launch a local network using docker compose:

```bash
docker-compose -f scripts/docker-compose.yml up -d
```

The `scripts` directory contains multiple compose files which can be used to launch various network configurations. Launching with docker ensures parachains are registered to the relaychain automatically and will begin authoring blocks shortly after launch.  

To ensure your network is functioning properly:  
1) Confirm docker containers are running
```bash
docker ps
``````
- These container names should have a status of 'up':
    - launch
    - frontend
    - kylin-kibana   
    - kylin-es
 
2) Check the container logs
```bash 
docker logs launch
```
- If the network was launched successfully, you will see something similar to this:
![POLKADOT LAUNCH COMPLETE](./doc/imgs/polkadotLaunchComplete.png)
```bash 
docker logs frontend
```
- If successful, ```Accepting connections at http://localhost:3001``` will be displayed in the terminal.

### Interacting with your network

To connect to your local network's user interface, visit [polkadot.js](http://localhost:3001/#/explorer) on port 3001. Interacting with your network is detailed [below](#submit-data-request).

## 2) From Source

- You can launch a network from source in two ways:
  - Manually launch a relaychain and parachain then register the parachain
  - Use the [polkadot-launch](https://github.com/paritytech/polkadot-launch) utility to configure your network and register the parachain amoung other helpful functions

We will discuss both approaches in detail.

### Prerequisite for both approaches

- First, setup your [Rust development environment](https://substrate.dev/docs/en/knowledgebase/getting-started). Then,

- Checkout the kylin-collator and polkadot repositories and build the respective binaries:

  - kylin-collator

    ```bash
    git clone --recursive https://github.com/Kylin-Network/kylin-collator.git

    cd kylin-collator
    git submodule update --recursive --remote

    cargo build --release
    ```

  - polkadot

    ```bash
    git clone https://github.com/paritytech/polkadot.git

    cd polkadot
    cargo build --release
    ```

### 2a) Launch using Polkadot-Launch Configuration

The `polkadot-launch` utility allows you to launch your network seamlessly by providing a custom json configuration file.

- #### Install polkadot-launch

  - To install polkadot-launch, run:

    ```bash
    yarn global add polkadot-launch

    Check installation

    polkadot-launch --version

    ```

- #### Define the configuration file

  Once we have the `polkadot-launch` utility installed, we need to define the configuration file.

  A configuration file has been provided within the repository at `scripts/polkadot-launch/kylinLaunchConfig.json`. You can customize it based on your requirements.

  - There are two sections in the file which are essential: `relaychain` and `parachains` 
  - relaychain: 3 key parameters
    - bin: Specify the location of the polkadot binary (in our case we built polkadot in a step above, so we can find the binary at `target/release/polkadot` in the polkadot directory). You can provide a relative or absolute path of the binary.
    - chain: Specify the type of the chain (in our case we will use rococo-local as we are launching a local network)
    - nodes: Configure the number of validators you want to have (we've added six validators, feel free to change per your requirement. The rule of thumb is at least two validators per collator). You can pass the name of the validators, additional flags, and both the tcp and websocket ports to be exposed.

  - parachains: 4 key parameters
    - bin: Specify the location of the kylin-collator binary
    - id: Specify the Para ID for the chain
    - balance: Set initial balance of your parachain
    - nodes: Node configurations for your parachain

- #### Launch the network

```bash
polkadot-launch scripts/polkadot-launch/kylinLaunchConfig.json
```

If the launch is successful, you will see:

```bash
🚀 POLKADOT LAUNCH COMPLETE 🚀
```

### 2b) Launch manually

If the prerequisites have been completed and we have our binaries as shown [above](#prerequisite-for-both-approaches), follow the steps below to launch the network.
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

#### Start a collator

```bash
# Customize the --chain flag for the path to your 'rococo-local.json' file
./target/release/kylin-collator --alice --collator --force-authoring --parachain-id 2000 --base-path cumulus_relay/kylin-collator --port 40333 --ws-port 8844 -- --execution wasm --chain <path to 'rococo-local.json' file> --port 30343 --ws-port 9942

./target/release/kylin-collator --alice --collator --force-authoring --parachain-id 2013 --base-path cumulus_relay/kylin-collator --port 40334 --ws-port 8845 -- --execution wasm --chain <path to 'rococo-local.json' file> --port 30344 --ws-port 9943

```

#### Start a collator on pichiu/rococo

```bash
target/release/kylin-collator --collator --bootnodes /ip4/35.78.250.13/tcp/40333/p2p/12D3KooWQ3stLjQa4R1Rrccw1s9ViZHna37iuosaAcS2bmzUn9oe  --unsafe-ws-external  --name pichiu-collator-<your id>  --force-authoring --parachain-id 2102 --chain ./pichiu-rococo-parachain-2102.json --port 40333 --ws-port 8844 --rpc-cors all --log parachain:debug  -- --execution wasm --chain ./rococo.json --port 30343 --ws-port 9977
```

- You should see your collator running and peering with the already running relay chain validators.  
- Your parachain will not begin authoring blocks until you have registered it on the relay chain.

#### Connect to user interface

1. You can connect to your network's user interface using a secure frontend connection like [apps](https://github.com/Kylin-Network/apps) or our pre-built `frontend` docker container.
2. Fill in config for our types in `Settings` -> `Developer`

```js
{
  "Address": "MultiAddress",
  "LookupSource": "MultiAddress",
  "DataRequest": {
    "para_id": "Option<ParaId>",
    "account_id": "Option<AccountId>",
    "requested_block_number": "BlockNumber",
    "processed_block_number": "Option<BlockNumber>",
    "requested_timestamp": "u128",
    "processed_timestamp": "Option<u128>",
    "payload": "Text",
    "feed_name": "Text",
    "is_query": "bool",
    "url": "Option<Text>"
  }
}
```
**NOTE:** If you launched your network using docker or polkadot-launch, your parachains should be automatically registered to the relaychain and you can skip the below two steps and can continue [here](#submit-data-request).
   
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
   - Navigate to the collator's custom endpoint 9942
   - Select `Network` -> `Explorer`
   - New blocks are being created if the value of `best` and `finalized` are incrementing higher


## Submit Data Request
1. In your [polkadot.js](http://localhost:3001/#/explorer) user interface, go to the top-left corner and click the dropdown. Under the development tab, enter a custom endpoint to connect to a collator: `ws://127.0.0.1:9942` or `ws://127.0.0.1:9943`. 
2. Select `Developer` -> `Extrinsics`. Submit a price request using the `requestPriceFeed` extrinsic.
   ![submitting a price request](./doc/imgs/requestPriceFeed.png)
