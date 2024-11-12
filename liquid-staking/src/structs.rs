multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, PartialEq, Eq, Debug)]
pub struct DelegationContractInfo<M: ManagedTypeApi> {
    pub admin_address: ManagedAddress<M>,
    pub total_staked: BigUint<M>,
    pub delegation_contract_cap: BigUint<M>,
    pub nr_nodes: u64,
    pub apy: u64,
    pub total_staked_from_ls_contract: BigUint<M>,
    pub total_unstaked_from_ls_contract: BigUint<M>,
    pub eligible: bool,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, PartialEq, Eq, Debug)]
pub struct UnstakeTokenAttributes {
    pub unstake_epoch: u64,
    pub unbond_epoch: u64,
}

impl UnstakeTokenAttributes {
    pub fn new(unstake_epoch: u64, unbond_epoch: u64) -> Self {
        UnstakeTokenAttributes {
            unstake_epoch,
            unbond_epoch,
        }
    }
}

#[type_abi]
#[derive(
    ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, PartialEq, Eq, Debug,
)]
pub struct DelegatorSelection<M: ManagedTypeApi> {
    pub delegation_address: ManagedAddress<M>,
    pub amount: BigUint<M>,
    pub space_left: Option<BigUint<M>>, // None means unlimited
}

impl<M: ManagedTypeApi> DelegatorSelection<M> {
    pub fn new(
        delegation_address: ManagedAddress<M>,
        amount: BigUint<M>,
        space_left: Option<BigUint<M>>,
    ) -> Self {
        DelegatorSelection {
            delegation_address,
            amount,
            space_left,
        }
    }
}

#[type_abi]
#[derive(
    ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, PartialEq, Eq, Debug,
)]
pub struct DelegationContractSelectionInfo<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub score: BigUint<M>,
    pub total_staked: BigUint<M>,
    pub apy: u64,
    pub nr_nodes: u64,
    pub total_staked_from_ls_contract: BigUint<M>,
    pub space_left: Option<BigUint<M>>, // None means unlimited
}

#[type_abi]
#[derive(TopEncode, TopDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, PartialEq, Eq, Debug)]
pub struct ScoringConfig {
    // Node limits
    pub min_nodes: u64,
    pub max_nodes: u64,

    // APY limits
    pub min_apy: u64,
    pub max_apy: u64,

    // Scoring weights
    pub stake_weight: u64,
    pub apy_weight: u64,
    pub nodes_weight: u64,

    // Scoring constants
    pub max_score_per_category: u64,
    pub exponential_base: u64,
    pub apy_growth_multiplier: u64,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        ScoringConfig {
            min_nodes: 1,
            max_nodes: 100,
            min_apy: 500,
            max_apy: 1000,
            stake_weight: 40,
            apy_weight: 50,
            nodes_weight: 10,
            max_score_per_category: 1000,
            exponential_base: 3,
            apy_growth_multiplier: 4,
        }
    }
}
