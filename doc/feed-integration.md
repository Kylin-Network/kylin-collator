# Add Kylin-feed pallet into runtime

These document show how to integrate the kylin-feed into other parachain runtime.

1. Open the `runtime/Cargo.toml` configuration file in a text editor.

2. Locate the [dependencies] section and note how other pallets are imported.

3. Add kylin-feed pallet, please also add pallet-uniques because the kylin-feed pallet is depend on it.

   For example, add lines similar to the following:

   ```toml
   pallet-uniques = { git = "https://github.com/paritytech/substrate", default-features = false, branch = "polkadot-v0.9.30" }
   kylin-feed = { git = "https://github.com/Kylin-Network/kylin-collator", branch = "v0.9.30-rococo", default-features = false }
   ```

   **Please make sure the code version(specify with branch) is match with the substrate pallet version currently using in the runtime code.**

4. Add the `kylin-feed/std` features to the list of `features` to enable when compiling the runtime.

   ```toml
   [features]
   default = ["std"]
   std = [
     ...
     "pallet-aura/std",
     "pallet-balances/std",
     'pallet-uniques/std',
     "kylin-feed/std",
     ...
   ]
   ```

5. Implement the `Config` trait for the kylin-feed pallet.

   1. Open the `runtime/src/lib.rs` file in a text editor.

   2. Locate the last line of the Balances code block.
   3. Add the following code block for the kylin-feed pallet:

   ```rust
   parameter_types! {
       pub const CollectionDeposit: Balance = UNITS / 10; // 1 / 10 UNIT deposit to create asset class
       pub const ItemDeposit: Balance = UNITS / 1_000; // 1 / 1000 UNIT deposit to create asset instance
       pub const KeyLimit: u32 = 32;   // Max 32 bytes per key
       pub const ValueLimit: u32 = 64; // Max 64 bytes per value
       pub const UniquesMetadataDepositBase: Balance = deposit(1, 129);
       pub const AttributeDepositBase: Balance = deposit(1, 0);
       pub const DepositPerByte: Balance = deposit(0, 1);
       pub const UniquesStringLimit: u32 = 128;
   }
   
   impl pallet_uniques::Config for Runtime {
       type RuntimeEvent = RuntimeEvent;
       type CollectionId = u32;
       type ItemId = u32;
       type Currency = Balances;
       type ForceOrigin = frame_system::EnsureRoot<AccountId>;
       type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
       type Locker = kylin_feed::Pallet<Runtime>;
       type CollectionDeposit = CollectionDeposit;
       type ItemDeposit = ItemDeposit;
       type MetadataDepositBase = UniquesMetadataDepositBase;
       type AttributeDepositBase = AttributeDepositBase;
       type DepositPerByte = DepositPerByte;
       type StringLimit = UniquesStringLimit;
       type KeyLimit = KeyLimit;
       type ValueLimit = ValueLimit;
       type WeightInfo = ();
   }
   
   parameter_types! {
       pub const MaxRecursions: u32 = 10;
       pub const ResourceSymbolLimit: u32 = 10;
       pub const PartsLimit: u32 = 25;
       pub const MaxPriorities: u32 = 25;
       pub const CollectionSymbolLimit: u32 = 100;
       pub const MaxResourcesOnMint: u32 = 100;
   }
   impl kylin_feed::Config for Runtime {
       type RuntimeEvent = RuntimeEvent;
       type RuntimeOrigin = RuntimeOrigin;
       type MaxRecursions = MaxRecursions;
       type ResourceSymbolLimit = ResourceSymbolLimit;
       type PartsLimit = PartsLimit;
       type MaxPriorities = MaxPriorities;
       type CollectionSymbolLimit = CollectionSymbolLimit;
       type MaxResourcesOnMint = MaxResourcesOnMint;
       type XcmSender = XcmRouter;
   }
   ```

7. Add kylin-feed to the `construct_runtime!` macro.

   ```rust
   construct_runtime!(
   pub enum Runtime where
      Block = Block,
      NodeBlock = opaque::Block,
      UncheckedExtrinsic = UncheckedExtrinsic
    {
      /* --snip-- */
      Balances: pallet_balances,
      /*** Add Lines ***/
      Uniques: pallet_uniques,
      //Fix Feed index(167) so the query feedback can route back
      KylinFeed: kylin_feed  = 167, 
    }
   );
   ```

   
