# Add Kylin-reporter pallet into runtime

These document show how to integrate the kylin-reporter into other parachain runtime.

1. Open the `runtime/Cargo.toml` configuration file in a text editor.

2. Locate the [dependencies] section and note how other pallets are imported.

3. Copy an existing pallet dependency description and replace the pallet name with `kylin-reporter` to make the pallet available to the runtime.

   For example, add a line similar to the following:

   ```toml
   kylin-reporter = { git = "https://github.com/Kylin-Network/kylin-collator", branch = "v0.9.30-rococo", default-features = false }
   ```

   **Please make sure the code version(specify with branch) is match with the substrate pallet version currently using in the runtime code.**

4. Add the `kylin-reporter/std` features to the list of `features` to enable when compiling the runtime.

   ```toml
   [features]
   default = ["std"]
   std = [
     ...
     "pallet-aura/std",
     "pallet-balances/std",
     "kylin-reporter/std",
     ...
   ]
   ```

5. Implement the `Config` trait for the kylin-feed pallet.

   1. Open the `runtime/src/lib.rs` file in a text editor.
   2. Locate the last line of the Balances code block.
   3. Add the following code block for the kylin-reporter pallet:

   ```rust
   impl kylin_reporter::Config for Runtime {
       type Event = Event;
       type AuthorityId = kylin_reporter::crypto::TestAuthId;
       type Call = Call;
       type Origin = Origin;
       type XcmSender = XcmRouter;
       type WeightInfo = kylin_reporter::weights::SubstrateWeight<Runtime>;
       type Currency = Balances;
       type Members = ();
   }
   ```

6. Add kylin-reporter to the `construct_runtime!` macro.

   ```rust
   construct_runtime!(
   pub enum Runtime where
      Block = Block,
      NodeBlock = opaque::Block,
      UncheckedExtrinsic = UncheckedExtrinsic
    {
      /* --snip-- */
      Balances: pallet_balances,
      /*** Add a Line ***/
      KylinReporter: kylin_reporter,
    }
   );
   ```

   
