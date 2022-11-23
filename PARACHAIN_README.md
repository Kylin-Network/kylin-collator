Docker image    

Please use the latest code from https://github.com/Kylin-Network/kylin-collator to build the correct docker image.

Recommended tooling
For easy interactions with parachains involved in the demo process we recommend using our https://github.com/Kylin-Network/kylin-toolbox .

Parachain Nodes - all connected to a Rococo-local relay chain using Polkadot version 0.9.30
Oracle node
use pichiu runtime
enable alice, bob.. accounts
Reporter node
use kylin runtime (with reporter pallet) 
enable alice, bob.. accounts
Feed node
use kylin runtime (with feed pallet) 
enable alice, bob.. accounts
Environment Preparation    
Bring up a Rococo local relay chain and setup all tree nodes mentioned earlier as different parachains;
Note down all ParaIDs obtained for each parachain;
Establish HRMP bidirectional channels between Oracle node and Reporter node 
Establish HRMP bidirectional  channels between Oracle node and Feed node  
Insert signing keys into Oracle node and Reporter node







Note: change the ports(8833,8533) in following command to respective RPC ports in the Oracle node and Reporter node

Key and Url_Endpoint Hex data    

Now we begin with support for two keys:




Feed Node Workflow

Make API call in sequential order:
create_collection(metadata, max, symbol) Params:
metadata: any hex data ('0xaabbcc')
max: 256
symbol: any hex data ('0xaabbcc') Return:
collection_id: 0
Example:
:~/kylin-toolbox$ yarn start feed createCollection -p 'ws://10.2.3.102:8845' -m 0xaabbc -x 256 -s 0xaabbcc
create_feed(collection_id, oracle_paraid, key, url) Params:
collection_id: 0 (return from create_collection)
oracle_paraid: 2000 (parachain id of Oracle node) key: Key hex data
url: URL hex data Return:
nft_id: 0
Example:
:~/kylin-toolbox$ yarn start feed createFeed -p 'ws://10.2.3.102:8845' -c 7 -o 2102 -k 0x4343417069 -u 0x68747470733a2f2f6d696e2d6170692e63727970746f636f6d706172652e636f6d2f646174612f70726963653f6673796d3d627463267473796d733d75736474
query_feed(collection_id, nft_id) Params:
collection_id: 0
nft_id: 0 Return:
NULL

Example:
:~/kylin-toolbox$ yarn start feed queryFeed -p 'ws://10.2.3.102:8845' -c 7 -n 0
Reporter Node Workflow

Make API call in sequential order:
set_kylin_id(oracle_paraid) Params:
oracle_paraid: 2000 (parachain id of Oracle node) Return:
NULL
Example:
:~/kylin-toolbox$  yarn start reporter setkylinId -p 'ws://10.2.3.102:8846' -o 2101
submit_api(key, url) Params:
key: Key hex data url: URL hex data
Return:
NULL
Example: 
:~/kylin-toolbox$ yarn start reporter submitApi -p 'ws://10.2.3.102:8846' -k 0x4357417069 -u 0x68747470733a2f2f6d696e2d6170692e63727970746f636f6d706172652e636f6d2f646174612f70726963653f6673796d3d627463267473796d733d75736474


