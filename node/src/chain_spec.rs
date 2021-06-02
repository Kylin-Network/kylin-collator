use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use kylin_node_runtime::{AccountId, AuraId, Signature};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use std::str::FromStr;
use sc_service::{ChainType, Properties};
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
pub type ChainSpec = sc_service::GenericChainSpec<kylin_node_runtime::GenesisConfig, Extensions>;

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

pub fn local_testnet_config(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"Kylin Local Testnet",
		// ID
		"kylin_local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
				vec![
					hex!["7c11cea2901e72fe525d7335e99d48bdf8dea2a983ac92fa3ab20508a438af73"]
						.unchecked_into(),
					hex!["287f278af79ef7f1b2a2b3d5a7c76a047e248232d13f0a5ec744789a96dc824d"]
						.unchecked_into(),
					get_from_seed::<AuraId>("Alice"),
					get_from_seed::<AuraId>("Bob"),
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
				id,
			)
		},
		Vec::new(),
		None,
		Some("Kylin"),
		Some(kylin_properties()),
		Extensions {
			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
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
		ChainType::Local,
		move || {
			testnet_genesis(
				AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
				vec![
					hex!["7c11cea2901e72fe525d7335e99d48bdf8dea2a983ac92fa3ab20508a438af73"]
						.unchecked_into(),
					hex!["287f278af79ef7f1b2a2b3d5a7c76a047e248232d13f0a5ec744789a96dc824d"]
						.unchecked_into(),
					get_from_seed::<AuraId>("Alice"),
					get_from_seed::<AuraId>("Bob"),
				],
				vec![
					sp_runtime::AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
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
				id,
			)
		},
		Vec::new(),
		None,
		Some("Kylin"),
		Some(kylin_properties()),
		Extensions {
			relay_chain: environment.into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

pub fn development_environment_config(id: ParaId, environment: &str) -> ChainSpec {
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
				id,
			)
		},
		Vec::new(),
		None,
		Some("Kylin"),
		Some(kylin_properties()),
		Extensions {
			relay_chain: environment.into(), // You MUST set this to the correct network!
			para_id: id.into(),
		},
	)
}

pub fn rococo_test_net(id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		"Kylin Rococo Testnet",
		"kylin_rococo_testnet",
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
					get_account_id_from_seed::<sr25519::Public>("Bob"),
				],
				id,
			)
		},
		Vec::new(),
		None,
		Some("Kylin"),
		Some(kylin_properties()),
		Extensions {
			relay_chain: "rococo".into(),
			para_id: id.into(),
		},
	)
}

fn testnet_genesis(
	root_key: AccountId,
	initial_authorities: Vec<AuraId>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> kylin_node_runtime::GenesisConfig {
	kylin_node_runtime::GenesisConfig {
		frame_system: kylin_node_runtime::SystemConfig {
			code: kylin_node_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: kylin_node_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 100 << 60))
				.collect(),
		},
		pallet_sudo: kylin_node_runtime::SudoConfig { key: root_key },
		parachain_info: kylin_node_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_aura: kylin_node_runtime::AuraConfig {
			authorities: initial_authorities,
		},
		cumulus_pallet_aura_ext: Default::default(),
	}
}

