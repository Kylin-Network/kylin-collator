# Kylin Pallet Documentation

Extrinsics
1. Submit Data Signed
2. 
3. Submit Data Unsigned
4. Submit Data via API
5. XCM Submit Data via API
6. Submit Price Feed
7. Receive Response From Parachain
8. Clear Process Requests Unsigned
9. Clear API Queue Unsigned 



Kylin Pricing API
The purpose of the Kylin Pricing API is to retrieve consolidated exchanges rates for various cryptocurrencies across multiple rate providers. The current list of exchange rate providers are:
1. Bancor
2. Coinbase
3. Coingecko
4. Cryptocompare
5. Cryptowatch

Resource: prices
HTTP Method: GET
Query Parameter: currency_pairs

Sample Query


Sample Response

Write Price Feed to Postgress DB, store hash onchain

Database 
Table Specifications
