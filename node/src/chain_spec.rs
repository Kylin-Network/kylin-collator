use cumulus_primitives_core::ParaId;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use pichiu_runtime::constants::currency::PCHU;
use runtime_common::*;
use sp_runtime::{AccountId32};
use sc_telemetry::TelemetryEndpoints;
use hex_literal::hex;
use std::str::FromStr;
use sp_core::crypto::UncheckedInto;


/// Properties for Kylin.
pub fn kylin_properties() -> Properties {
	let mut properties = Properties::new();
	properties.insert("ss58Format".into(), 31.into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("tokenSymbol".into(), "KYL".into());
	properties
}

/// Specialized `ChainSpec` for the Pichiu parachain runtime.
pub type PichiuChainSpec = sc_service::GenericChainSpec<pichiu_runtime::GenesisConfig, Extensions>;
/// Specialized `ChainSpec` for the develop parachain runtime.
// pub type DevelopmentChainSpec = sc_service::GenericChainSpec<pichiu_runtime::GenesisConfig>;

const POLKADOT_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

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

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_from_seed::<AuraId>(seed)
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(keys: AuraId) -> pichiu_runtime::SessionKeys {
	pichiu_runtime::SessionKeys { aura: keys }
}

pub fn pichiu_local_network(id: ParaId) -> PichiuChainSpec {
	let mut properties = Properties::new();
	properties.insert("ss58Format".into(), 31_u8.into());
	properties.insert("tokenSymbol".into(), "PCHU".into());
	properties.insert("tokenDecimals".into(), 18_u8.into());

	PichiuChainSpec::from_genesis(
		"Pichiu Local Testnet",
		"pichiu_local_testnet",
		ChainType::Local,
		move || {
			pichiu_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), get_collator_keys_from_seed("Alice")),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), get_collator_keys_from_seed("Bob")),
				],
				endowed_accounts_local(),
				Some(50000000 * PCHU),
				id,
				30_000_000 * PCHU
			)
		},
		Vec::new(),
		None,
		Some("Pichiu"),
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo-local".into(),
            para_id: id.into(),
        },
	)
}

pub fn pichiu_development_network(id: ParaId) -> PichiuChainSpec {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "PCHU".into());
	properties.insert("tokenDecimals".into(), 18_u8.into());

	PichiuChainSpec::from_genesis(
		"Pichiu Testnet",
		"pichiu_testnet",
		ChainType::Live,
		move || {
			pichiu_genesis(
				AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
				vec![
					(AccountId32::from_str("7c11cea2901e72fe525d7335e99d48bdf8dea2a983ac92fa3ab20508a438af73").unwrap(), get_collator_keys_from_seed("validator_key")),
					(AccountId32::from_str("287f278af79ef7f1b2a2b3d5a7c76a047e248232d13f0a5ec744789a96dc824d").unwrap(), get_collator_keys_from_seed("validator_key")),
				],
				endowed_accounts(),
				Some(50000000 * PCHU),
				id,
				30_000_000 * PCHU
			)
		},
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(POLKADOT_TELEMETRY_URL.to_string(), 0)])
				.expect("Polkadot telemetry url is valid; qed"),
		),
		Some("Pichiu"),
		None,
		Some(properties),
		Extensions {
			relay_chain: "westend".into(),
            para_id: id.into(),
        },
	)
}

pub fn pichiu_network(id: ParaId) -> PichiuChainSpec {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "PCHU".into());
	properties.insert("tokenDecimals".into(), 18_u8.into());

	PichiuChainSpec::from_genesis(
		"Pichiu Network",
		"pichiu_Network",
		ChainType::Live,
		move || {
			pichiu_genesis(
				AccountId32::from_str("5Gn1igfpf4hP7iG1Gsm1AbwPBCpR8BmHK4b6i2VrGHQS1kAJ").unwrap(),
				vec![
					(AccountId32::from_str("7c11cea2901e72fe525d7335e99d48bdf8dea2a983ac92fa3ab20508a438af73").unwrap(), get_collator_keys_from_seed("validator_key")),
					(AccountId32::from_str("287f278af79ef7f1b2a2b3d5a7c76a047e248232d13f0a5ec744789a96dc824d").unwrap(), get_collator_keys_from_seed("validator_key")),
				],
				endowed_accounts(),
				Some(50000000 * PCHU),
				id,
				30_000_000 * PCHU
			)
		},
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(POLKADOT_TELEMETRY_URL.to_string(), 0)])
				.expect("Polkadot telemetry url is valid; qed"),
		),
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "kusama".into(),
            para_id: id.into(),
        },
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
	initial_authorities: Vec<(AccountId, AuraId)>,
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
				.to_vec()
		},
		balances: pichiu_runtime::BalancesConfig { balances },
		sudo: pichiu_runtime::SudoConfig { key: Some(root_key) },
		council:pichiu_runtime::CouncilConfig{
			members: Default::default(),
			phantom: Default::default(),
		},
		// scheduler: pichiu_runtime::SchedulerConfig {},
		vesting: Default::default(),
		crowdloan_rewards: pichiu_runtime::CrowdloanRewardsConfig {
			funded_amount: crowdloan_fund_pot,
		},
		parachain_info: pichiu_runtime::ParachainInfoConfig { parachain_id: id },
		// collator_selection: pichiu_runtime::CollatorSelectionConfig {
		// 	invulnerables: initial_authorities.iter().cloned().map(|(acc, _)| acc).collect(),
		// 	candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
		// 	..Default::default()
		// },
		// session: pichiu_runtime::SessionConfig { // validator session
        //     keys: initial_authorities
        //         .iter()
        //         .cloned()
        //         .map(|(acc, aura)| {
        //             (
        //                 acc.clone(),                // account id
        //                 acc,                        // validator id
        //                 get_dev_session_keys(aura), // session keys
        //             )
        //         })
        //         .collect(),
        // },
		aura_ext: Default::default(),
		aura: pichiu_runtime::AuraConfig {
            authorities: Default::default(),
        },
		parachain_system: Default::default(),
	}
}