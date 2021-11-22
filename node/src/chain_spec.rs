use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use kylin_collator_runtime::{AccountId, AuraId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use std::str::FromStr;
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use serde::{Deserialize, Serialize};

use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::{AccountId32, traits::{IdentifyAccount, Verify}};


/// Properties for Kylin.
pub fn kylin_properties() -> Properties {
	let mut properties = Properties::new();
	properties.insert("ss58Format".into(), 31.into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("tokenSymbol".into(), "KYL".into());

	properties
}

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<kylin_collator_runtime::GenesisConfig>;
/// Specialized `ChainSpec` for the shell parachain runtime.
pub type ShellChainSpec = sc_service::GenericChainSpec<shell_runtime::GenesisConfig, Extensions>;


const POLKADOT_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
	where
		AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn shell_testnet_genesis(parachain_id: ParaId) -> shell_runtime::GenesisConfig {
	shell_runtime::GenesisConfig {
		system: shell_runtime::SystemConfig {
			code: shell_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		parachain_info: shell_runtime::ParachainInfoConfig { parachain_id },
		parachain_system: Default::default(),
	}
}


pub fn get_shell_chain_spec(id: ParaId) -> ShellChainSpec {
	ShellChainSpec::from_genesis(
		"Shell Local Testnet",
		"shell_local_testnet",
		ChainType::Local,
		move || shell_testnet_genesis(id),
		vec![],
		None,
		Some("Kylin"),
		Some(kylin_properties()),
		Extensions {
			relay_chain: "westend".into(),
			para_id: id.into(),
		},
	)
}

pub fn local_environment_config(id: ParaId, environment: &str) -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		format!("kylin {} testnet", environment).as_str(),
		// ID
		format!("kylin-{}-testnet", environment).as_str(),
		ChainType::Development,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					get_from_seed::<AuraId>("Alice"),
					get_from_seed::<AuraId>("Bob"),
					hex!["7c11cea2901e72fe525d7335e99d48bdf8dea2a983ac92fa3ab20508a438af73"]
					.unchecked_into(),
					hex!["287f278af79ef7f1b2a2b3d5a7c76a047e248232d13f0a5ec744789a96dc824d"]
					.unchecked_into()
				],
				vec![
					AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				endowed_accounts_local(),
				id,
			)
		},
		Vec::new(),
		None,
		Some("Kylin"),
		Some(kylin_properties()),
		Default::default()
	)
}

pub fn development_environment_config(id: ParaId,environment: &str) -> ChainSpec {
	ChainSpec::from_genesis(
		format!("kylin {} testnet", environment).as_str(),
		// ID
		format!("kylin-{}-testnet", environment).as_str(),
		ChainType::Live,
		move || {
			testnet_genesis(
				AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
				vec![
					hex!["7c11cea2901e72fe525d7335e99d48bdf8dea2a983ac92fa3ab20508a438af73"]
						.unchecked_into(),
					hex!["287f278af79ef7f1b2a2b3d5a7c76a047e248232d13f0a5ec744789a96dc824d"]
						.unchecked_into(),
				],
				vec![
					AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
					get_account_id_from_seed::<sr25519::Public>("Alice"),
				],
				id,
			)
		},
		Vec::new(),
		Some(
			TelemetryEndpoints::new(vec![(POLKADOT_TELEMETRY_URL.to_string(), 0)])
				.expect("Polkadot telemetry url is valid; qed"),
		),
		Some("Kylin"),
		Some(kylin_properties()),
		Default::default()
	)
}

pub fn pichiu_local_network(para_id: ParaId) -> PichiuChainSpec {
	let mut properties = Properties::new();
	properties.insert("ss58Format".into(), 31.into());
	properties.insert("tokenSymbol".into(), "PCHU".into());
	properties.insert("tokenDecimals".into(), 18.into());

	PichiuChainSpec::from_genesis(
		"Pichiu Local Testnet",
		"pichiu_local_testnet",
		ChainType::Local,
		move || {
			pichiu_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					get_from_seed::<AuraId>("Alice"),
					get_from_seed::<AuraId>("Bob"),
				],
				endowed_accounts_local(),
				Some(50000000 * PCHU),
				para_id,
				30_000_000 * PCHU
			)
		},
		Vec::new(),
		None,
		Some("Pichiu"),
		Some(properties),
		Default::default(),
	)
}

pub fn pichiu_development_network(para_id: ParaId) -> PichiuChainSpec {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "PCHU".into());
	properties.insert("tokenDecimals".into(), 18.into());

	PichiuChainSpec::from_genesis(
		"Pichiu Testnet",
		"pichiu_testnet",
		ChainType::Live,
		move || {
			pichiu_genesis(
				AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
				vec![
					hex!["7c11cea2901e72fe525d7335e99d48bdf8dea2a983ac92fa3ab20508a438af73"]
					.unchecked_into(),
					hex!["287f278af79ef7f1b2a2b3d5a7c76a047e248232d13f0a5ec744789a96dc824d"]
					.unchecked_into()
				],
				endowed_accounts(),
				Some(50000000 * PCHU),
				para_id,
				30_000_000 * PCHU
			)
		},
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(POLKADOT_TELEMETRY_URL.to_string(), 0)])
				.expect("Polkadot telemetry url is valid; qed"),
		),
		Some("Pichiu"),
		Some(properties),
		Default::default(),
	)
}

pub fn pichiu_network(para_id: ParaId) -> PichiuChainSpec {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "PCHU".into());
	properties.insert("tokenDecimals".into(), 18.into());

	PichiuChainSpec::from_genesis(
		"Pichiu Network",
		"pichiu_Network",
		ChainType::Live,
		move || {
			pichiu_genesis(
				AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
				vec![
					hex!["7c11cea2901e72fe525d7335e99d48bdf8dea2a983ac92fa3ab20508a438af73"]
					.unchecked_into(),
					hex!["287f278af79ef7f1b2a2b3d5a7c76a047e248232d13f0a5ec744789a96dc824d"]
					.unchecked_into()
				],
				endowed_accounts(),
				Some(50000000 * PCHU),
				para_id,
				30_000_000 * PCHU
			)
		},
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(POLKADOT_TELEMETRY_URL.to_string(), 0)])
				.expect("Polkadot telemetry url is valid; qed"),
		),
		None,
		Some(properties),
		Default::default(),
	)
}

fn endowed_accounts() -> Vec<AccountId> {
	vec![
		AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
	]
}


fn endowed_accounts_local() -> Vec<AccountId> {
	vec![
		AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Charlie"),
		get_account_id_from_seed::<sr25519::Public>("Dave"),
		get_account_id_from_seed::<sr25519::Public>("Eve"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
		get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
		get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
		get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
	]
}

fn pichiu_genesis(
	root_key: AccountId,
	initial_authorities: Vec<pichiu_runtime::AuraId>,
	endowed_accounts: Vec<pichiu_runtime::AccountId>,
	total_issuance: Option<pichiu_runtime::Balance>,
	id: ParaId,
	crowdloan_fund_pot: Balance,
) -> pichiu_runtime::GenesisConfig {
	let num_endowed_accounts = endowed_accounts.len();
	let balances = match total_issuance {
		Some(total_issuance) => {
			let balance_per_endowed = total_issuance
				.checked_div(num_endowed_accounts as pichiu_runtime::Balance)
				.unwrap_or(0 as pichiu_runtime::Balance);
			endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, balance_per_endowed))
				.collect()
		}
		None => vec![],
	};

	pichiu_runtime::GenesisConfig {
		system: pichiu_runtime::SystemConfig {
			code: pichiu_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: pichiu_runtime::BalancesConfig { balances },
		sudo: pichiu_runtime::SudoConfig { key: root_key },
		scheduler: pichiu_runtime::SchedulerConfig {},
		vesting: Default::default(),
		crowdloan_rewards: pichiu_runtime::CrowdloanRewardsConfig {
			funded_amount: crowdloan_fund_pot,
		},
		parachain_info: pichiu_runtime::ParachainInfoConfig { parachain_id: id },

		aura_ext: Default::default(),
		aura: pichiu_runtime::AuraConfig {
			authorities: initial_authorities,
		},
		parachain_system: Default::default(),
	}
}
fn development_genesis(
	root_key: AccountId,
	initial_authorities: Vec<AuraId>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> kylin_collator_runtime::GenesisConfig {
	let _initial_balance: u128 = 1_000_000_000 * 1000;

	kylin_collator_runtime::GenesisConfig {
		system: kylin_collator_runtime::SystemConfig {
			code: kylin_collator_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: kylin_collator_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 10 << 60))
				.collect(),
		},
		sudo: kylin_collator_runtime::SudoConfig { key: root_key },
		parachain_info: kylin_collator_runtime::ParachainInfoConfig { parachain_id: id },
		aura: kylin_collator_runtime::AuraConfig {
			authorities: initial_authorities,
		},
	
		aura_ext: Default::default(),
		parachain_system: Default::default(),
	}
}