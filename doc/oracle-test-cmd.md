## Launch local test-net

```bash
// In kylin-collator root directory
cd scripts/parachain-launch/output/
docker-compose up -d --build
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
./dist/main.js feed createCollection
./dist/main.js feed createFeed -c 0 -o 2000 -k 'PriceEthUsdt' -u 'https://min-api.cryptocompare.com/data/price?fsym=eth&tsyms=usdt' -v '/USDT'
./dist/main.js feed queryFeed -c 0 -n 0
```

**Test Oracle APIs**

```bash
./dist/main.js oracle submitApi -k 'PriceEtcUsdt' -u 'https://min-api.cryptocompare.com/data/price?fsym=etc&tsyms=usdt' -v '/USDT'
```

