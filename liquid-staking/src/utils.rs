use crate::{
    structs::{
        ClaimStatus, ClaimStatusType, DelegationContractData, DelegationContractInfo,
        DelegatorSelection,
    },
    ERROR_BAD_DELEGATION_ADDRESS, ERROR_CLAIM_EPOCH, ERROR_CLAIM_START, ERROR_FAILED_TO_DISTRIBUTE,
    ERROR_FIRST_DELEGATION_NODE, ERROR_MINIMUM_ROUNDS_NOT_PASSED, ERROR_NO_DELEGATION_CONTRACTS,
    ERROR_OLD_CLAIM_START, MIN_EGLD_TO_DELEGATE,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const DECIMALS: usize = 18;
pub const MAX_PROVIDERS: usize = 25;
pub const PERCENTAGE_TOTAL: u64 = 10_000; // 100%

#[multiversx_sc::module]
pub trait UtilsModule:
    crate::storage::StorageModule
    + crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn get_delegation_contract_for_delegate(
        &self,
        amount_to_delegate: &BigUint,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        self.get_delegation_contract(
            amount_to_delegate,
            |contract_data, amount_per_provider| {
                contract_data.delegation_contract_cap == BigUint::zero()
                    || &contract_data.delegation_contract_cap - &contract_data.total_staked
                        >= *amount_per_provider
            },
            |selected_addresses, amount_to_delegate, min_egld, total_stake| {
                self.distribute_amount(
                    selected_addresses,
                    amount_to_delegate,
                    min_egld,
                    total_stake,
                    true,
                )
            },
        )
    }

    fn get_delegation_contract_for_undelegate(
        &self,
        amount_to_undelegate: &BigUint,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        self.get_delegation_contract(
            amount_to_undelegate,
            |contract_data, _| {
                contract_data.total_staked_from_ls_contract > BigUint::from(MIN_EGLD_TO_DELEGATE)
            },
            |selected_addresses, amount_to_undelegate, min_egld, total_stake| {
                self.distribute_amount(
                    selected_addresses,
                    amount_to_undelegate,
                    min_egld,
                    total_stake,
                    false,
                )
            },
        )
    }

    fn get_delegation_contract<F, D>(
        &self,
        amount: &BigUint,
        filter_fn: F,
        distribute_fn: D,
    ) -> ManagedVec<DelegatorSelection<Self::Api>>
    where
        F: Fn(&DelegationContractData<Self::Api>, &BigUint) -> bool,
        D: Fn(
            &ManagedVec<DelegationContractInfo<Self::Api>>,
            &BigUint,
            &BigUint,
            &BigUint,
        ) -> ManagedVec<DelegatorSelection<Self::Api>>,
    {
        require!(
            !self.delegation_addresses_list().is_empty(),
            ERROR_NO_DELEGATION_CONTRACTS
        );

        let min_egld = BigUint::from(MIN_EGLD_TO_DELEGATE);
        let max_providers = self.calculate_max_providers(amount, &min_egld);
        let amount_per_provider = amount / &BigUint::from(max_providers as u64);

        let mut selected_addresses = ManagedVec::new();
        let mut total_stake = BigUint::zero();

        for delegation_address_node in self.delegation_addresses_list().iter().take(max_providers) {
            let delegation_address = delegation_address_node.get_value_as_ref();
            let contract_data = self.delegation_contract_data(delegation_address).get();

            if filter_fn(&contract_data, &amount_per_provider) {
                selected_addresses.push(DelegationContractInfo {
                    address: delegation_address.clone(),
                    total_staked: contract_data.total_staked.clone(),
                    total_staked_from_ls_contract: contract_data
                        .total_staked_from_ls_contract
                        .clone(),
                    space_left: if contract_data.delegation_contract_cap == BigUint::zero() {
                        None
                    } else {
                        Some(&contract_data.delegation_contract_cap - &contract_data.total_staked)
                    },
                });
                total_stake += &contract_data.total_staked_from_ls_contract;
            }
        }

        require!(!selected_addresses.is_empty(), ERROR_BAD_DELEGATION_ADDRESS);

        distribute_fn(&selected_addresses, amount, &min_egld, &total_stake)
    }

    fn distribute_amount(
        &self,
        selected_addresses: &ManagedVec<DelegationContractInfo<Self::Api>>,
        amount: &BigUint,
        min_egld: &BigUint,
        total_stake: &BigUint,
        is_delegate: bool,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        let mut result = ManagedVec::new();
        let mut remaining_amount = amount.clone();

        if is_delegate {
            let amount_per_provider = amount / &BigUint::from(selected_addresses.len() as u64);
            if total_stake == &BigUint::zero() {
                for contract_info in selected_addresses.iter() {
                    result.push(DelegatorSelection::new(
                        contract_info.address.clone(),
                        amount_per_provider.clone(),
                        contract_info.space_left.clone(),
                    ));
                    remaining_amount -= &amount_per_provider;
                }
            } else {
                let inverse_total_stake: BigUint<Self::Api> = selected_addresses
                    .iter()
                    .fold(BigUint::zero(), |acc, info| {
                        acc + (total_stake - &info.total_staked_from_ls_contract)
                    });

                if inverse_total_stake == BigUint::zero() {
                    // Distribute equally if inverse_total_stake is zero
                    for contract_info in selected_addresses.iter() {
                        result.push(DelegatorSelection::new(
                            contract_info.address.clone(),
                            amount_per_provider.clone(),
                            contract_info.space_left.clone(),
                        ));
                        remaining_amount -= &amount_per_provider;
                    }
                } else {
                    for contract_info in selected_addresses.iter() {
                        let inverse_stake =
                            total_stake - &contract_info.total_staked_from_ls_contract;

                        let inverse_stake_ratio = inverse_stake * amount / &inverse_total_stake;

                        let mut amount_to_delegate =
                            inverse_stake_ratio.min(remaining_amount.clone());

                        if let Some(space_left) = &contract_info.space_left {
                            amount_to_delegate = amount_to_delegate.min(space_left.clone());
                        }

                        amount_to_delegate = amount_to_delegate.max(min_egld.clone());

                        if amount_to_delegate <= remaining_amount {
                            result.push(DelegatorSelection::new(
                                contract_info.address.clone(),
                                amount_to_delegate.clone(),
                                contract_info.space_left.clone(),
                            ));
                            remaining_amount -= &amount_to_delegate;
                        }
                    }
                }
            }
        } else {
            for contract_info in selected_addresses.iter() {
                let proportion =
                    &contract_info.total_staked_from_ls_contract * amount / total_stake;
                let amount_to_undelegate = proportion
                    .max(min_egld.clone())
                    .min(remaining_amount.clone());

                if amount_to_undelegate > BigUint::zero() {
                    result.push(DelegatorSelection::new(
                        contract_info.address.clone(),
                        amount_to_undelegate.clone(),
                        Some(contract_info.total_staked_from_ls_contract.clone()),
                    ));
                    remaining_amount -= &amount_to_undelegate;
                }

                if remaining_amount == BigUint::zero() {
                    break;
                }
            }
        }

        if remaining_amount > BigUint::zero() {
            for (index, delegator_selection) in result.clone().iter().enumerate() {
                let available_space = match &delegator_selection.space_left {
                    Some(space_left) => space_left - &delegator_selection.amount,
                    None => remaining_amount.clone(),
                };

                if available_space > BigUint::zero() {
                    let amount_to_add = remaining_amount.clone().min(available_space);
                    let new_amount = &delegator_selection.amount + &amount_to_add;

                    let _ = result.set(
                        index,
                        &DelegatorSelection::new(
                            delegator_selection.delegation_address.clone(),
                            new_amount,
                            delegator_selection.space_left.clone(),
                        ),
                    );

                    remaining_amount -= &amount_to_add;

                    if remaining_amount == BigUint::zero() {
                        break;
                    }
                }
            }
        }

        require!(
            remaining_amount == BigUint::zero(),
            ERROR_FAILED_TO_DISTRIBUTE
        );

        result
    }

    fn calculate_max_providers(
        &self,
        amount_to_delegate: &BigUint<Self::Api>,
        min_egld: &BigUint<Self::Api>,
    ) -> usize {
        let amount_decimal =
            ManagedDecimal::<Self::Api, ConstDecimals<DECIMALS>>::from(amount_to_delegate.clone());
        let min_egld_decimal =
            ManagedDecimal::<Self::Api, ConstDecimals<DECIMALS>>::from(min_egld.clone());

        let max_providers_decimal = amount_decimal / min_egld_decimal;
        let max_providers_biguint = max_providers_decimal.trunc();

        let max_providers_limit = BigUint::from(MAX_PROVIDERS as u64);
        let max_providers = max_providers_biguint.min(max_providers_limit);

        max_providers.to_u64().unwrap() as usize
    }

    fn calculate_instant_amount(
        &self,
        sent_amount: &BigUint,
        pending_amount: &BigUint,
        min_amount: &BigUint,
    ) -> BigUint {
        if pending_amount <= min_amount {
            return BigUint::zero();
        }

        let max_instant = sent_amount - min_amount;

        if max_instant <= pending_amount - min_amount {
            max_instant
        } else {
            pending_amount - min_amount
        }
    }

    fn check_claim_operation(
        &self,
        current_claim_status: &ClaimStatus,
        old_claim_status: ClaimStatus,
        current_epoch: u64,
    ) {
        require!(
            current_claim_status.status == ClaimStatusType::None
                || current_claim_status.status == ClaimStatusType::Pending,
            ERROR_CLAIM_START
        );
        require!(
            old_claim_status.status == ClaimStatusType::Redelegated
                || old_claim_status.status == ClaimStatusType::Insufficent,
            ERROR_OLD_CLAIM_START
        );
        require!(
            current_epoch > old_claim_status.last_claim_epoch,
            ERROR_CLAIM_EPOCH
        );
    }

    fn prepare_claim_operation(&self, current_claim_status: &mut ClaimStatus, current_epoch: u64) {
        if current_claim_status.status == ClaimStatusType::None {
            let delegation_addresses_mapper = self.delegation_addresses_list();
            require!(
                delegation_addresses_mapper.front().unwrap().get_node_id() != 0,
                ERROR_FIRST_DELEGATION_NODE
            );
            current_claim_status.status = ClaimStatusType::Pending;
            current_claim_status.last_claim_epoch = current_epoch;
            current_claim_status.current_node =
                delegation_addresses_mapper.front().unwrap().get_node_id();
        }
    }

    fn calculate_split(&self, total_amount: &BigUint, cut_percentage: &BigUint) -> BigUint {
        total_amount * cut_percentage / PERCENTAGE_TOTAL
    }

    fn require_min_rounds_passed(&self) {
        let block_round = self.blockchain().get_block_round();
        let rounds_per_epoch = self.rounds_per_epoch().get();
        let minimum_rounds = self.minimum_rounds().get();

        require!(
            rounds_per_epoch - block_round <= minimum_rounds,
            ERROR_MINIMUM_ROUNDS_NOT_PASSED
        );
    }
}
