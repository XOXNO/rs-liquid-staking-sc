use crate::{
    structs::{
        ClaimStatus, ClaimStatusType, DelegationContractInfo, DelegationContractSelectionInfo,
        DelegatorSelection,
    },
    ERROR_BAD_DELEGATION_ADDRESS, ERROR_CLAIM_EPOCH, ERROR_CLAIM_START, ERROR_FAILED_TO_DISTRIBUTE,
    ERROR_FIRST_DELEGATION_NODE, ERROR_NO_DELEGATION_CONTRACTS, ERROR_OLD_CLAIM_START,
    MIN_EGLD_TO_DELEGATE,
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
                (contract_data.delegation_contract_cap == BigUint::zero()
                    || &contract_data.delegation_contract_cap - &contract_data.total_staked
                        >= *amount_per_provider)
                    && contract_data.eligible
            },
            |selected_addresses,
             amount_to_delegate,
             min_egld,
             total_stake,
             total_nodes,
             total_apy| {
                self.distribute_amount(
                    selected_addresses,
                    amount_to_delegate,
                    min_egld,
                    total_stake,
                    total_nodes,
                    total_apy,
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
            |contract_data, amount_per_provider| {
                contract_data.total_staked_from_ls_contract >= amount_per_provider.clone()
            },
            |selected_addresses,
             amount_to_undelegate,
             min_egld,
             total_stake,
             total_nodes,
             total_apy| {
                self.distribute_amount(
                    selected_addresses,
                    amount_to_undelegate,
                    min_egld,
                    total_stake,
                    total_nodes,
                    total_apy,
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
        F: Fn(&DelegationContractInfo<Self::Api>, &BigUint) -> bool,
        D: Fn(
            &mut ManagedVec<DelegationContractSelectionInfo<Self::Api>>,
            &BigUint,
            &BigUint,
            &BigUint,
            u64,
            u64,
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
        let mut total_nodes = 0;
        let mut total_apy = 0;

        for delegation_address_node in self.delegation_addresses_list().iter().take(max_providers) {
            let delegation_address = delegation_address_node.get_value_as_ref();
            let contract_data = self.delegation_contract_data(delegation_address).get();

            if filter_fn(&contract_data, &amount_per_provider) {
                total_stake += &contract_data.total_staked_from_ls_contract;
                total_nodes += contract_data.nr_nodes;
                total_apy += contract_data.apy;

                selected_addresses.push(DelegationContractSelectionInfo {
                    address: delegation_address.clone(),
                    space_left: if contract_data.delegation_contract_cap == BigUint::zero() {
                        None
                    } else {
                        Some(&contract_data.delegation_contract_cap - &contract_data.total_staked)
                    },
                    total_staked: contract_data.total_staked,
                    apy: contract_data.apy,
                    score: BigUint::zero(),
                    nr_nodes: contract_data.nr_nodes,
                    total_staked_from_ls_contract: contract_data.total_staked_from_ls_contract,
                });
            }
        }

        require!(!selected_addresses.is_empty(), ERROR_BAD_DELEGATION_ADDRESS);

        distribute_fn(
            &mut selected_addresses,
            amount,
            &min_egld,
            &total_stake,
            total_nodes,
            total_apy,
        )
    }

    fn distribute_amount(
        &self,
        selected_addresses: &mut ManagedVec<DelegationContractSelectionInfo<Self::Api>>,
        amount: &BigUint,
        min_egld: &BigUint,
        total_stake: &BigUint,
        total_nodes: u64,
        total_apy: u64,
        is_delegate: bool,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        let mut result = ManagedVec::new();
        let mut remaining_amount = amount.clone();

        let total_score = self.update_selected_addresses_scores(
            selected_addresses,
            is_delegate,
            total_stake,
            total_apy,
            total_nodes,
            min_egld,
        );

        let amount_per_provider = amount / &BigUint::from(selected_addresses.len() as u64);

        for index in 0..selected_addresses.len() {
            if remaining_amount == BigUint::zero() {
                break;
            }

            let contract_info = selected_addresses.get(index);

            // If total stake is zero or total score is zero, distribute equally
            if (total_stake == &BigUint::zero() || total_score == BigUint::zero()) && is_delegate {
                remaining_amount -= &amount_per_provider;
                result.push(DelegatorSelection::new(
                    contract_info.address,
                    amount_per_provider.clone(),
                    contract_info.space_left,
                ));
                continue;
            }

            let proportion = contract_info.score * amount / &total_score;

            // Ensure the amount is not greater than the remaining amount
            let mut amount_to_delegate = proportion.min(remaining_amount.clone());

            // If there is a space left, ensure the amount is not greater than the space left
            if let Some(space_left) = &contract_info.space_left {
                amount_to_delegate = amount_to_delegate.min(space_left.clone());
            }

            // Ensure the amount is at least the minimum EGLD to delegate
            amount_to_delegate = amount_to_delegate.max(min_egld.clone());

            if !is_delegate {
                // Ensure that in case of undelegation, the amount is not greater than the total staked from the LS contract
                amount_to_delegate = amount_to_delegate.min(contract_info.total_staked_from_ls_contract.clone());
            }
            remaining_amount -= &amount_to_delegate;

            result.push(DelegatorSelection::new(
                contract_info.address,
                amount_to_delegate,
                if is_delegate {
                    contract_info.space_left
                } else {
                    Some(contract_info.total_staked_from_ls_contract)
                },
            ));
        }

        // In case of rounding dust due to math
        // Most of the time this will add the remaining amount to the first provider
        self._distribute_remaining_amount(&mut result, &mut remaining_amount);

        result
    }

    fn _distribute_remaining_amount(
        &self,
        result: &mut ManagedVec<DelegatorSelection<Self::Api>>,
        remaining_amount: &mut BigUint,
    ) {
        // In case of rounding dust due to math
        // Most of the time this will add the remaining amount to the first provider
        if *remaining_amount > BigUint::zero() {
            for index in 0..result.len() {
                let delegator_selection = result.get(index);
                let available_space = match &delegator_selection.space_left {
                    Some(space_left) => space_left - &delegator_selection.amount,
                    None => remaining_amount.clone(),
                };

                if available_space > BigUint::zero() {
                    let amount_to_add = available_space.min(remaining_amount.clone());
                    let new_amount = &delegator_selection.amount + &amount_to_add;

                    let _ = result.set(
                        index,
                        &DelegatorSelection::new(
                            delegator_selection.delegation_address,
                            new_amount,
                            delegator_selection.space_left,
                        ),
                    );

                    *remaining_amount -= &amount_to_add;

                    if *remaining_amount == BigUint::zero() {
                        break;
                    }
                }
            }
            require!(
                *remaining_amount == BigUint::zero(),
                ERROR_FAILED_TO_DISTRIBUTE
            );
        }
    }

    fn calculate_and_update_score(
        &self,
        info: &mut DelegationContractSelectionInfo<Self::Api>,
        is_delegate: bool,
        total_stake: &BigUint,
        total_apy: u64,
        total_nodes: u64,
        min_egld: &BigUint,
    ) -> BigUint {
        let inverse_stake_score = if is_delegate && total_stake > &BigUint::zero() {
            total_stake - &info.total_staked_from_ls_contract
        } else {
            info.total_staked_from_ls_contract.clone()
        };

        let apy_score = if is_delegate {
            BigUint::from(info.apy).mul(min_egld)
        } else {
            BigUint::from(total_apy - info.apy).mul(min_egld)
        };

        let node_score = if is_delegate {
            BigUint::from(total_nodes - info.nr_nodes).mul(min_egld)
        } else {
            BigUint::from(info.nr_nodes).mul(min_egld)
        };

        let score = inverse_stake_score + apy_score + node_score;
        info.score = score.clone();
        score
    }

    fn update_selected_addresses_scores(
        &self,
        selected_addresses: &mut ManagedVec<DelegationContractSelectionInfo<Self::Api>>,
        is_delegate: bool,
        total_stake: &BigUint,
        total_apy: u64,
        total_nodes: u64,
        min_egld: &BigUint,
    ) -> BigUint {
        let mut total_score = BigUint::zero();

        for index in 0..selected_addresses.len() {
            let mut info = selected_addresses.get(index);
            let score = self.calculate_and_update_score(
                &mut info,
                is_delegate,
                total_stake,
                total_apy,
                total_nodes,
                min_egld,
            );
            total_score += &score;
            let _ = selected_addresses.set(index, &info);
        }

        total_score
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
        if pending_amount <= min_amount || sent_amount <= min_amount {
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
        old_claim_status: &ClaimStatus,
        current_epoch: u64,
    ) {
        require!(
            current_claim_status.status == ClaimStatusType::None
                || current_claim_status.status == ClaimStatusType::Pending,
            ERROR_CLAIM_START
        );

        require!(
            current_epoch > old_claim_status.last_claim_epoch,
            ERROR_CLAIM_EPOCH
        );

        require!(
            old_claim_status.status == ClaimStatusType::Redelegated
                || old_claim_status.status == ClaimStatusType::Insufficent,
            ERROR_OLD_CLAIM_START
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
        // TODO: Implement once new hooks are available in the VM with the future mainnet upgrade
        return;
        // let block_round = self.blockchain().get_block_round();
        // let rounds_per_epoch = self.rounds_per_epoch().get();
        // let minimum_rounds = self.minimum_rounds().get();

        // require!(
        //     rounds_per_epoch - block_round <= minimum_rounds,
        //     ERROR_MINIMUM_ROUNDS_NOT_PASSED
        // );
    }
}
