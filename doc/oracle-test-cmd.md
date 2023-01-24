## Launch local test-net

```bash
// In kylin-collator root directory
cd scripts/parachain-launch/output/
docker-compose up -d --build
```

**insert node keys**

```bash
curl http://localhost:8833 -H "Content-Type:application/json;charset=utf-8" -X POST  --data '{
  "jsonrpc":"2.0",
  "id":1,
  "method":"author_insertKey",
  "params": [
    "ocpf",
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Bob",
    "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"
  ]
}'
curl http://localhost:8533 -H "Content-Type:application/json;charset=utf-8" -X POST  --data '{
  "jsonrpc":"2.0",
  "id":1,
  "method":"author_insertKey",
  "params": [
    "ocrp",
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Bob",
    "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"
  ]
}'
```

## Use kylin-toolbox to test pallet APIs

**build kylin-toolbox**

```bash
// In kylin-toolbox root directory
yarn
yarn build
```

**Set relaychain and parachains ENV value**

```bash
export RCHAIN_WS=ws://127.0.0.1:9944
export PCHAIN0_WS=ws://127.0.0.1:8844
export PCHAIN1_WS=ws://127.0.0.1:8544
```

**Establish duplex HRMP channels between parachains**

```bash
./dist/main.js oracle open 2000 2010
./dist/main.js oracle open 2010 2000
```

**Test Reporter APIs**

```bash
./dist/main.js reporter setkylinId -o 2000
./dist/main.js reporter submitApi -k 'PriceBtcUsdt' -u 'https://min-api.cryptocompare.com/data/price?fsym=Btc&tsyms=usdt' -v '/USDT'
```

**Test Feed APIs**

```bash
./dist/main.js feed createCollection -o 2000
./dist/main.js feed createFeed -o 2000 -c 0 -k 'PriceEthUsdt' -u 'https://min-api.cryptocompare.com/data/price?fsym=eth&tsyms=usdt' -v '/USDT'
./dist/main.js feed queryFeed -o 2000 -c 0 -n 0
./dist/main.js feed queryFeedByKey -o 2000 -k 'PriceEthUsdt'
```

**Test Oracle APIs**

```bash
./dist/main.js oracle submitApi -k 'PriceEtcUsdt' -u 'https://min-api.cryptocompare.com/data/price?fsym=etc&tsyms=usdt' -v '/USDT'
```

