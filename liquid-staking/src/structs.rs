multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone)]
pub enum ClaimStatusType {
    None,
    Pending,
    Finished,
    Redelegated,
    Insufficent,
}

#[derive(NestedEncode, NestedDecode, TopEncode, TopDecode, PartialEq, Eq, TypeAbi, Clone)]
pub struct ClaimStatus {
    pub status: ClaimStatusType,
    pub last_claim_epoch: u64,
    pub current_node: u32,
}

impl Default for ClaimStatus {
    fn default() -> Self {
        Self {
            status: ClaimStatusType::None,
            last_claim_epoch: 0,
            current_node: 0,
        }
    }
}

#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug,
)]
pub struct DelegationContractData<M: ManagedTypeApi> {
    pub admin_address: ManagedAddress<M>,
    pub total_staked: BigUint<M>,
    pub delegation_contract_cap: BigUint<M>,
    pub nr_nodes: u64,
    pub apy: u64,
    pub total_staked_from_ls_contract: BigUint<M>,
    pub total_unstaked_from_ls_contract: BigUint<M>,
    pub eligible: bool,
}

#[derive(
    TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Eq, Debug,
)]
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

#[derive(
    ManagedVecItem,
    TopEncode,
    TopDecode,
    NestedEncode,
    NestedDecode,
    TypeAbi,
    Clone,
    PartialEq,
    Eq,
    Debug,
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

#[derive(
    ManagedVecItem,
    TopEncode,
    TopDecode,
    NestedEncode,
    NestedDecode,
    TypeAbi,
    Clone,
    PartialEq,
    Eq,
    Debug,
)]
pub struct DelegationContractInfo<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub total_staked: BigUint<M>,
    pub total_staked_from_ls_contract: BigUint<M>,
    pub space_left: Option<BigUint<M>>, // None means unlimited
}

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}
