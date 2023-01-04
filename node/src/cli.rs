
use std::{
	fs,
	io::{self, Write},
	path::PathBuf,
};
use sc_chain_spec::ChainSpec;
use sp_core::hexdisplay::HexDisplay;
use sp_runtime::{
	traits::{Block as BlockT},
	StateVersion,
};
use codec::Encode;
use cumulus_client_cli::generate_genesis_block;

/// Sub-commands supported by the collator.
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
	/// Build a chain specification.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Validate blocks.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Export blocks.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the state of a given block into a chain spec.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Revert the chain to a previous state.
	Revert(sc_cli::RevertCmd),

	/// Remove the whole chain.
	PurgeChain(cumulus_client_cli::PurgeChainCmd),

	/// Export the genesis state of the parachain.
	#[clap(name = "export-genesis-state")]
	ExportGenesisState(ExportGenesisStateCommand),

	/// Export the genesis wasm of the parachain.
	#[clap(name = "export-genesis-wasm")]
	ExportGenesisWasm(cumulus_client_cli::ExportGenesisWasmCommand),

	/// The custom benchmark subcommmand benchmarking runtime pallets.
	#[clap(subcommand)]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),
	
	/// Try some testing command against a specified runtime state.
	#[cfg(feature = "try-runtime")]
	TryRuntime(try_runtime_cli::TryRuntimeCmd),

	/// Errors since the binary was not build with `--features try-runtime`.
	#[cfg(not(feature = "try-runtime"))]
	TryRuntime,
}

/// Command for exporting the genesis state of the parachain
#[derive(Debug, clap::Parser)]
pub struct ExportGenesisStateCommand {
	/// Output file name or stdout if unspecified.
	#[clap(action)]
	pub output: Option<PathBuf>,

	/// Id of the parachain this state is for.
    ///
    /// Default: 100
    #[structopt(long)]
    pub parachain_id: Option<u32>,

	/// Write output in binary. Default is to write in hex.
	#[clap(short, long)]
	pub raw: bool,

	#[allow(missing_docs)]
	#[clap(flatten)]
	pub shared_params: sc_cli::SharedParams,
}

impl ExportGenesisStateCommand {
	/// Run the export-genesis-state command
	pub fn run<Block: BlockT>(
		&self,
		chain_spec: &dyn ChainSpec,
		genesis_state_version: StateVersion,
	) -> sc_cli::Result<()> {
		let block: Block = generate_genesis_block(chain_spec, genesis_state_version)?;
		let raw_header = block.header().encode();
		let output_buf = if self.raw {
			raw_header
		} else {
			format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
		};

		if let Some(output) = &self.output {
			fs::write(output, output_buf)?;
		} else {
			io::stdout().write_all(&output_buf)?;
		}

		Ok(())
	}
}

impl sc_cli::CliConfiguration for ExportGenesisStateCommand {
	fn shared_params(&self) -> &sc_cli::SharedParams {
		&self.shared_params
	}
}

#[derive(Debug, clap::Parser)]
#[clap(
	propagate_version = true,
	args_conflicts_with_subcommands = true,
	subcommand_negates_reqs = true
)]
pub struct Cli {
	#[clap(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[clap(flatten)]
	pub run: RunCmd,

	/// Disable automatic hardware benchmarks.
	///
	/// By default these benchmarks are automatically ran at startup and measure
	/// the CPU speed, the memory bandwidth and the disk speed.
	///
	/// The results are then printed out in the logs, and also sent as part of
	/// telemetry, if telemetry is enabled.
	#[clap(long)]
	pub no_hardware_benchmarks: bool,

	/// Relay chain arguments
	#[clap(raw = true)]
	pub relay_chain_args: Vec<String>,
}

#[derive(Debug, clap::Parser)]
pub struct RunCmd {
    #[clap(flatten)]
	pub base: cumulus_client_cli::RunCmd,

    /// Id of the parachain this collator collates for.
	#[clap(long)]
	pub parachain_id: Option<u32>,
}

impl std::ops::Deref for RunCmd {
	type Target = cumulus_client_cli::RunCmd;

	fn deref(&self) -> &Self::Target {
		&self.base
	}
}

#[derive(Debug)]
pub struct RelayChainCli {
	/// The actual relay chain cli object.
	pub base: polkadot_cli::RunCmd,

	/// Optional chain id that should be passed to the relay chain.
	pub chain_id: Option<String>,

	/// The base path that should be used by the relay chain.
	pub base_path: Option<PathBuf>,
}

impl RelayChainCli {
	/// Parse the relay chain CLI parameters using the para chain `Configuration`.
	pub fn new<'a>(
		para_config: &sc_service::Configuration,
		relay_chain_args: impl Iterator<Item = &'a String>,
	) -> Self {
		let extension = crate::chain_spec::Extensions::try_get(&*para_config.chain_spec);
		let chain_id = extension.map(|e| e.relay_chain.clone());
		let base_path = para_config.base_path.as_ref().map(|x| x.path().join("polkadot"));
		Self { base_path, chain_id, base: clap::Parser::parse_from(relay_chain_args) }
	}
}
