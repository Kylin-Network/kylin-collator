#!/usr/bin/env bash
#Key	Value
#Secret phrase	clip organ olive upper oak void inject side suit toilet stick narrow
#Secret seed	0x4bd2b2c1dad3dbe3fa37dc6ad5a4e32ddd8ad84b938179ac905b0622880e86e7
#SR25519
#Public key	0x9effc1668ca381c242885516ec9fa2b19c67b6684c02a8a3237b6862e5c8cd7e
#SS58 Address	5FfBQ3kwXrbdyoqLPvcXRp7ikWydXawpNs2Ceu3WwFdhZ8W4
#ED25519
#Public key	0xb48004c6e1625282313b07d1c9950935e86894a2e4f21fb1ffee9854d180c781
#SS58 Address	5G9NWJ5P9uk7a**4yCKeLZJqXWW6hjuMyRJDmw4ofqxG8Js2
# insert aura key
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -X POST  --data '{
  "jsonrpc":"2.0",
  "id":1,
  "method":"author_insertKey",
  "params": [
    "aura",
    "clip organ olive upper oak void inject side suit toilet stick narrow",
    "0x9effc1668ca381c242885516ec9fa2b19c67b6684c02a8a3237b6862e5c8cd7e"
  ]
}'

# insert gran key
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -X POST  --data '{
  "jsonrpc":"2.0",
  "id":1,
  "method":"author_insertKey",
  "params": [
    "gran",
    "clip organ olive upper oak void inject side suit toilet stick narrow",
    "0xb48004c6e1625282313b07d1c9950935e86894a2e4f21fb1ffee9854d180c781"
  ]
}'

# insert ocw price_fetch key
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -X POST  --data '{
  "jsonrpc":"2.0",
  "id":1,
  "method":"author_insertKey",
  "params": [
    "ocpf",
    "clip organ olive upper oak void inject side suit toilet stick narrow",
    "0x9effc1668ca381c242885516ec9fa2b19c67b6684c02a8a3237b6862e5c8cd7e"
  ]
}'

# insert ocw data_fetch key
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -X POST  --data '{
  "jsonrpc":"2.0",
  "id":1,
  "method":"author_insertKey",
  "params": [
    "dftc",
    "clip organ olive upper oak void inject side suit toilet stick narrow",
    "0x9effc1668ca381c242885516ec9fa2b19c67b6684c02a8a3237b6862e5c8cd7e"
  ]
}'
