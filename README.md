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

### Interact
Using [Kylin Front End](https://github.com/Kylin-Network/kylin-front-end) which can be used to interact with Kylin Node.

``` bash
git clone https://github.com/Kylin-Network/kylin-front-end.git
cd ./kylin-front-end
yarn install
```


## Run

### Development Chain

Purge any existing dev chain state:

```bash
./target/debug/kylin-node purge-chain --dev
```

Start a dev chain:

```bash
./target/debug/kylin-node --dev
```

Or, start a dev chain with detailed logging:

```bash
RUST_LOG=debug RUST_BACKTRACE=1 ./target/debug/kylin-node -lruntime=debug --dev
```

### Use `release` version

Replace `debug` with `release`.

**Caution! Donot try to run `release` version everytime, it will take lots of time.**


### Using polkadot.js webUI
visit <https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/settings/developer>

type config in Settings>>Developer.
```js
{
  "Address": "AccountId",
  "LookupSource": "AccountId",
  "DataInfo": {
    "url": "Text",
    "data": "Text"
  }
}
```

#### Add ocw signer
run script.
```bash
./scripts/insert_alice_key.sh
```
if the signer has not enough balance, please charge money.

#### Add a request url
select Developer>>Extrinsics, then using priceFetchModule.addFetchDataRequest(url), type a url encode hex format.
![pic](doc/imgs/addFetchDataRequest.png)

#### query requested data
select Developer>>Chain state, then using priceFetchModule.requestedOffchainData(u64), press +.
![pic](doc/imgs/queryRequestedData.jpg)
