# Docker image

Please use latest code in v0.9.28-rococo to build docker image

# Node

- Oracle node
  - use **pichiu** runtime
  - enable alice, bob.. accouts
- Reporter node
  - use **kylin** runtime  (with reporter pallet)
  - enable alice, bob.. accouts
- Feed node
  - use **kylin** runtime (with feed pallet)
  - enable alice, bob.. accouts

# Environment Preparation

- **Establish HRMP channel between Orcale node and Reporter node**
- **Establish HRMP channel between Orcale node and Feed node**

- **Insert signing keys into Orcale node and Reporter node**

  **change the ports(8833,8533)** in following command to respective RPC ports in the Orcale node and Reporter node

  ```
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
# Key and Url_Endpoint Hex data

**We begin with support for two keys:**

```bash
Key: "CCApi":
keyHex = 0x4343417069
API: "https://min-api.cryptocompare.com/data/price?fsym=btc&tsyms=usdt"
apiHex = 0x68747470733a2f2f6d696e2d6170692e63727970746f636f6d706172652e636f6d2f646174612f70726963653f6673796d3d627463267473796d733d75736474

Key: "CWApi":
keyHex = 0x4357417069
API: "https://min-api.cryptocompare.com/data/price?fsym=btc&tsyms=usdt"
apiHex = 0x68747470733a2f2f6d696e2d6170692e63727970746f636f6d706172652e636f6d2f646174612f70726963653f6673796d3d627463267473796d733d75736474
```
# Feed Node Work flow

**Make API call in sequential order:** 

- create_collection(metadata, max, symbol) 

  - Params:
    - metadata: any hex data ('0xaabbcc')
    - max: 256
    - symbol: any hex data ('0xaabbcc')
  - Return:
    -  collection_id: 0
  - **Example:**
    - **create_collection('0xaabbcc', 256, '0xaabbcc')** 
- create_feed(collection_id, oracle_paraid, key, url)
  - Params:
    - collection_id: 0 (return from *create_collection*)
    - oracle_paraid: 2000 (parachain id of Oracle node)
    - key: Key hex data
    - url: URL hex data
    
  - Return:

    - nft_id: 0

  - **Example:**
- **create_feed(0, 2000 , '0x4343417069', '0x68747470733a2f2f6d696e2d6170692e63727970746f636f6d706172652e636f6d2f646174612f70726963653f6673796d3d627463267473796d733d75736474')** 
  
- query_feed(collection_id, nft_id) 

  - Params:
    - collection_id: 0
    - nft_id: 0
  - Return:
    
    -  NULL
  - **Example:**
    - **query_feed(0, 0)** 

# Reporter Node Work flow

**Make API call in sequential order:** 

- set_kylin_id(oracle_paraid) 

  - Params:
    - oracle_paraid: 2000 (parachain id of Oracle node)
  - Return:
    -  NULL
  - **Example:**
    - **set_kylin_id(2000)** 
- submit_api(key, url)
  - Params:
    - key: Key hex data
    - url: URL hex data

  - Return:

    - NULL

  - **Example:**
- **submit_api('0x4357417069', '0x68747470733a2f2f6d696e2d6170692e63727970746f636f6d706172652e636f6d2f646174612f70726963653f6673796d3d627463267473796d733d75736474')** 
  


