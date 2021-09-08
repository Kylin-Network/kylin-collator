# Kylin Collator

Using this repository, you can choose to build and launch the network in multiple ways.

First is to use [Docker](https://www.docker.com/). Composing up with Docker will automatically launch a local network containing multiple validators (polkadot nodes) and collators (kylin collators) as well as the full user interface on port 3001.

The other way is to build your network from source which again has two ways to do so and is discussed step by step below

## 1) Using Docker

Follow these steps to prepare a local development environment :hammer_and_wrench:

### Setup

You will need to install [Docker](https://www.docker.com/products/docker-desktop) to launch our pre-built containers.

**NOTE:** You can not only launch your network with Docker, but can also build from source if you prefer.

### Run

Launching the network using docker compose:

```bash
docker-compose -f scripts/docker-compose.yml up -d
```

The `scripts` directory contains multiple compose files which can be used to launch various network configurations and uses `docker-compose.yml` to launch a local network containing a relaychain with six validators and two kylin parachains. Both parachains are registered to the relaychain and will begin authoring blocks shortly after launch.

Ensure your network is functioning properly
Confirm docker containers are running.
```bash
docker ps
``````
- These container names should have a status of 'up':
    - init-kibana-dashboard
    - insert-ocw
    - kylin-kibana   
    - launch
    - frontend
    - kylin-es
 
Check the container logs:
```bash 
docker logs launch
```
- If the network was launched successfully, you will see something similar to this:
![POLKADOT LAUNCH COMPLETE](./doc/imgs/polkadotLaunchComplete.png)
```bash 
docker logs frontend
```

### Testing

Testing for all the sections has been shown [below](#interact-and-testing )

## 2) From Source

- There are two ways you can build from source
  - Manually launching relaychains, parachains and registering it
  - Using the polkadot-launch utility and the configuration file

We will discuss both the approaches in detail.

### Prerequisite for both the approaches

- First, setup your [Rust development environment](https://substrate.dev/docs/en/knowledgebase/getting-started). Then,

- Checkout the code from github and build as shown below

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

we can use the `polkadot-launch` utility script to launch our network very simply by provding the custom json configuration file.

- #### Install polkadot-launch

  - We need to have the polkadot launch node script file installed, please run the below command in your terminal

    ```bash
    yarn global add polkadot-launch

    Check installation

    polkadot-launch --version

    ```

- #### Defining the configuration file

  Once we have the `polkadot-launch` utility installed, we need to define the configuration file.

  The configuration files has been provided within the repository inside `scrips-->polkadot_launch-->kylinPolkadotLaunchConfig.json`, you can customize it based on your requirement.

  - There are two sections in the file which are essential: `relaychain` and the `parachains`
  - relaychain: Here we have 3 key parameters to take care

    - bin: Specify the location of the relaychain binary(in our case we build the polkadot in the above step so we can find the binary in the folder `target/release/`). Make sure to provide the absolute path of the binary
    - chain: Specify the type of the chain(in our case we will be using rococo-local as we are launching the local network)
    - nodes: You can specify the number of validators you need to have and their configurations( we have used six validators, feel free to have as per your requirement. But make sure you have atleast two validators). It has name of the validators with some flags and both tcp and websocket exposed port

  - parachains: We have to provide 4 paramters
    - bin: Specify the location of the binary of your parachain
    - id: Specify the Para ID for the chain
    - balance: Set initial balance for some of the know account(like alice, bob etc)
    - nodes: Nodes configurations for the parachains

- #### Launching the network

```bash
polkadot-launch scripts/polkadot_launch/kylinPolkadotLaunchConfig.json
```

You must wait for the prompt

```bash
🚀 POLKADOT LAUNCH COMPLETE 🚀
```

### 2b) Launch manually

Make sure that the prerequisite have been completed and we have built the binaries as shown [above](#prerequisite-for-both-the-approaches). Follow the steps below to launch the network
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

- You should see your collator running and peering with the already running relay chain validators.
- Your parachain will not begin authoring blocks until you have registered it on the relay chain.

## Interact and Testing 

1. You can either Connect to polkadot.js using a secure frontend connection like [apps](https://github.com/Kylin-Network/apps) or our pre-built `frontend` Docker container.
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
    "url": "Text"
  }
}
```
3. If you followed and launched the network either using the docker or polkadot-launch then you should have your parachains registered to the relaychain and you can skip the below two steps and can go [here](#submit-data-request)
   
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


#### Submit data request
1.  In the UI, you can go to the top-left corner and click into the dropdown. Make sure you have the development option selected. You need to go to the custom under `Development` -> `Custom`. Enter the url which is `ws://127.0.0.1:9942` or `ws://127.0.0.1:9943` 
2. Submit a price request using the `requestPriceFeed` extrinsic
   ![example of submitting a price request](./doc/imgs/requestPriceFeed.png)
