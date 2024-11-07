use crate::{
    structs::{
        DelegationContractInfo, DelegationContractSelectionInfo, DelegatorSelection, ScoringConfig,
    },
    StorageCache, ERROR_BAD_DELEGATION_ADDRESS, ERROR_FAILED_TO_DISTRIBUTE, ERROR_NOT_MANAGER,
    ERROR_NO_DELEGATION_CONTRACTS, ERROR_SCORING_CONFIG_NOT_SET, MIN_EGLD_TO_DELEGATE,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const DECIMALS: usize = 18;

pub const BPS: u64 = 10_000; // 100%

#[multiversx_sc::module]
pub trait UtilsModule:
    crate::storage::StorageModule
    + crate::config::ConfigModule
    + crate::events::EventsModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn is_manager(&self, address: &ManagedAddress, required: bool) -> bool {
        let owner = self.blockchain().get_owner_address();
        let is_manager = self.managers().contains(address) || address == &owner;
        if required && !is_manager {
            sc_panic!(ERROR_NOT_MANAGER);
        }
        is_manager
    }

    fn get_delegation_contract_for_delegate(
        &self,
        amount_to_delegate: &BigUint,
        storage_cache: &mut StorageCache<Self>,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        self.get_delegation_contract(
            amount_to_delegate,
            |contract_data, amount_per_provider, all_providers_limit_per_provider| {
                contract_data.eligible
                    && (contract_data.delegation_contract_cap == BigUint::zero()
                        || &contract_data.delegation_contract_cap - &contract_data.total_staked
                            >= *amount_per_provider)
                    || (&contract_data.delegation_contract_cap - &contract_data.total_staked
                        >= *all_providers_limit_per_provider)
                    || (&contract_data.delegation_contract_cap - &contract_data.total_staked
                        >= BigUint::from(MIN_EGLD_TO_DELEGATE))
            },
            |selected_addresses,
             amount_to_delegate,
             min_egld,
             total_stake,
             total_nodes,
             total_apy,
             storage_cache| {
                self.distribute_amount(
                    selected_addresses,
                    amount_to_delegate,
                    min_egld,
                    total_stake,
                    total_nodes,
                    total_apy,
                    true,
                    storage_cache,
                )
            },
            storage_cache,
        )
    }

    fn get_delegation_contract_for_undelegate(
        &self,
        amount_to_undelegate: &BigUint,
        storage_cache: &mut StorageCache<Self>,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        self.get_delegation_contract(
            amount_to_undelegate,
            |contract_data, amount_per_provider, all_providers_limit_per_provider| {
                &contract_data.total_staked_from_ls_contract >= amount_per_provider
                    // Allow some flexibility in case the amount is not exactly the same as the one requested but can still be unstaked from the contract
                    // In case of remaining amount, the next transaction will pick it up
                    || (&(&contract_data.total_staked_from_ls_contract / &BigUint::from(2u64))
                        >= amount_per_provider)
                        || (&contract_data.total_staked_from_ls_contract
                        >= all_providers_limit_per_provider)
            },
            |selected_addresses,
             amount_to_undelegate,
             min_egld,
             total_stake,
             total_nodes,
             total_apy,
             storage_cache| {
                self.distribute_amount(
                    selected_addresses,
                    amount_to_undelegate,
                    min_egld,
                    total_stake,
                    total_nodes,
                    total_apy,
                    false,
                    storage_cache,
                )
            },
            storage_cache,
        )
    }

    fn get_delegation_contract<F, D>(
        &self,
        amount: &BigUint,
        filter_fn: F,
        distribute_fn: D,
        storage_cache: &mut StorageCache<Self>,
    ) -> ManagedVec<DelegatorSelection<Self::Api>>
    where
        F: Fn(&DelegationContractInfo<Self::Api>, &BigUint, &BigUint) -> bool,
        D: Fn(
            &mut ManagedVec<DelegationContractSelectionInfo<Self::Api>>,
            &BigUint,
            &BigUint,
            &BigUint,
            u64,
            u64,
            &mut StorageCache<Self>,
        ) -> ManagedVec<DelegatorSelection<Self::Api>>,
    {
        let map_list = self.delegation_addresses_list();
        require!(!map_list.is_empty(), ERROR_NO_DELEGATION_CONTRACTS);

        let min_egld = BigUint::from(MIN_EGLD_TO_DELEGATE);
        let max_providers = self.calculate_max_providers(amount, &min_egld, map_list.len());
        let amount_per_provider = amount / &BigUint::from(max_providers as u64);
        let all_providers_limit_per_provider = amount / &BigUint::from(map_list.len() as u64);

        let mut selected_addresses = ManagedVec::new();
        let mut total_stake = BigUint::zero();
        let mut total_nodes = 0;
        let mut total_apy = 0;

        for delegation_address in map_list.iter() {
            let contract_data = self.delegation_contract_data(&delegation_address).get();

            if filter_fn(
                &contract_data,
                &amount_per_provider,
                &all_providers_limit_per_provider,
            ) {
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

            if selected_addresses.len() == max_providers {
                break;
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
            storage_cache,
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
        storage_cache: &mut StorageCache<Self>,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        let mut result = ManagedVec::new();
        let mut remaining_amount = amount.clone();

        let config = self.get_scoring_config();
        let total_score = self.update_selected_addresses_scores(
            selected_addresses,
            is_delegate,
            total_stake,
            total_apy,
            total_nodes,
            min_egld,
            &config,
        );

        for index in 0..selected_addresses.len() {
            if remaining_amount == BigUint::zero() || remaining_amount < *min_egld {
                break;
            }

            let contract_info = selected_addresses.get(index);

            let proportion = contract_info.score * amount / &total_score;

            // Ensure the amount is not greater than the remaining amount
            let mut amount_to_delegate = proportion.min(remaining_amount.clone());

            // Ensure the amount is at least the minimum EGLD to delegate or undelegation
            amount_to_delegate = amount_to_delegate.max(min_egld.clone());

            if is_delegate {
                // If there is a space left, ensure the amount is not greater than the space left
                if let Some(space_left) = &contract_info.space_left {
                    amount_to_delegate = amount_to_delegate.min(space_left.clone());
                }
            } else {
                // Ensure that in case of undelegation, the amount is not greater than the total staked from the LS contract
                amount_to_delegate =
                    amount_to_delegate.min(contract_info.total_staked_from_ls_contract.clone());
                let left_over_amount =
                    &contract_info.total_staked_from_ls_contract - &amount_to_delegate;
                // If the left over amount is less than the required minimum or not zero, skip provider
                if left_over_amount < BigUint::from(MIN_EGLD_TO_DELEGATE)
                    && left_over_amount > BigUint::zero()
                {
                    continue;
                }
            }

            // If the amount is less than the minimum EGLD to delegate or undelegation, skip provider
            if amount_to_delegate < *min_egld {
                continue;
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

        // In case of rounding dust due to math or unavialable providers to cover the entire amount in one transaction
        // Most of the time this will add the remaining amount to the first provider
        self._distribute_remaining_amount(
            &mut result,
            &mut remaining_amount,
            is_delegate,
            min_egld,
            storage_cache,
        );

        require!(!result.is_empty(), ERROR_BAD_DELEGATION_ADDRESS);

        result
    }

    fn _distribute_remaining_amount(
        &self,
        result: &mut ManagedVec<DelegatorSelection<Self::Api>>,
        remaining_amount: &mut BigUint,
        is_delegate: bool,
        min_egld: &BigUint,
        storage_cache: &mut StorageCache<Self>,
    ) {
        // In case of rounding dust due to math
        // Most of the time this will add the remaining amount to the first provider
        if *remaining_amount > BigUint::zero() {
            for index in 0..result.len() {
                let delegator_selection = result.get(index);
                let available_space = match &delegator_selection.space_left {
                    Some(space_left) => {
                        if space_left < &delegator_selection.amount {
                            continue;
                        } else {
                            space_left - &delegator_selection.amount
                        }
                    }
                    None => remaining_amount.clone(),
                };

                if available_space > BigUint::zero() {
                    let amount_to_add = available_space.clone().min(remaining_amount.clone());
                    if !is_delegate {
                        let left_over_amount = &available_space - &amount_to_add;
                        // If the left over amount is less than the required minimum or not zero, skip provider
                        if left_over_amount < *min_egld && left_over_amount > BigUint::zero() {
                            continue;
                        }
                    }
                    let new_amount = &delegator_selection.amount + &amount_to_add;

                    let _ = result.set(
                        index,
                        DelegatorSelection::new(
                            delegator_selection.delegation_address,
                            new_amount,
                            delegator_selection.space_left,
                        ),
                    );

                    *remaining_amount -= amount_to_add;

                    if *remaining_amount == BigUint::zero() {
                        break;
                    }
                }
            }

            if *remaining_amount >= *min_egld {
                // We can arrive here when for example we undelegate 20k EGLD and the entire 20k is not fitting in the first batch of providers
                // In this case we need to add the remaining amount to the pending EGLD back and the next transaction will pick it up over a new batch of providers
                // Both for delegate and undelegate
                if is_delegate {
                    storage_cache.pending_egld += remaining_amount.clone();
                    return;
                } else {
                    storage_cache.pending_egld_for_unstake += remaining_amount.clone();
                    return;
                }
            } else {
                require!(
                    *remaining_amount == BigUint::zero(),
                    ERROR_FAILED_TO_DISTRIBUTE
                );
            }
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
        config: &ScoringConfig,
    ) -> BigUint {
        let node_score = self.calculate_node_score(info.nr_nodes, total_nodes, is_delegate, config);
        let apy_score = self.calculate_apy_score(info.apy, total_apy, is_delegate, config);
        let stake_score = self.calculate_stake_score(
            &info.total_staked_from_ls_contract,
            total_stake,
            is_delegate,
        );

        let final_score = self.combine_scores(node_score, apy_score, stake_score, min_egld, config);
        info.score = final_score.clone();
        final_score
    }

    fn calculate_node_score(
        &self,
        nr_nodes: u64,
        total_nodes: u64,
        is_delegate: bool,
        config: &ScoringConfig,
    ) -> BigUint {
        let absolute_score = self.calculate_absolute_score(
            config,
            nr_nodes,
            config.min_nodes,
            config.max_nodes,
            true, // always quadratic for better distribution
            is_delegate, // inverse for delegate (lower nodes → higher score)
                  // not inverse for undelegate (higher nodes → higher score)
        );

        let relative_score = self.calculate_relative_score(
            config,
            nr_nodes,
            total_nodes,
            true, // always quadratic for better distribution
            is_delegate, // inverse for delegate (lower nodes → higher score)
                  // not inverse for undelegate (higher nodes → higher score)
        );

        absolute_score + relative_score
    }

    fn calculate_apy_score(
        &self,
        apy: u64,
        total_apy: u64,
        is_delegate: bool,
        config: &ScoringConfig,
    ) -> BigUint {
        let absolute_score = self.calculate_absolute_score(
            config,
            apy,
            config.min_apy,
            config.max_apy,
            true, // always exponential for better distribution
            !is_delegate, // inverse for undelegate (lower APY → higher score)
                  // not inverse for delegate (higher APY → higher score)
        );

        let relative_score = self.calculate_relative_score(
            config,
            apy,
            total_apy,
            true, // always quadratic for better distribution
            !is_delegate, // inverse for undelegate (lower APY → higher score)
                  // not inverse for delegate (higher APY → higher score)
        );

        absolute_score + relative_score
    }

    fn calculate_stake_score(
        &self,
        staked: &BigUint,
        total_stake: &BigUint,
        is_delegate: bool,
    ) -> BigUint {
        let bps = BigUint::from(BPS);

        if total_stake == &BigUint::zero() {
            return BigUint::zero();
        }

        let stake_percentage = staked.mul(&bps) / total_stake;

        if is_delegate {
            if stake_percentage >= bps {
                BigUint::zero()
            } else {
                // Exponential reward for lower stake percentages
                let remaining_capacity = &bps - &stake_percentage;
                (remaining_capacity.pow(2)) / bps
            }
        } else {
            // Exponential reward for higher stake percentages when undelegate
            (stake_percentage.pow(2)) / bps
        }
    }

    fn combine_scores(
        &self,
        node_score: BigUint,
        apy_score: BigUint,
        stake_score: BigUint,
        min_egld: &BigUint,
        config: &ScoringConfig,
    ) -> BigUint {
        let weighted_score = node_score
            .mul(config.nodes_weight)
            .add(&apy_score.mul(config.apy_weight))
            .add(&stake_score.mul(config.stake_weight));

        weighted_score.div(100u64).mul(min_egld)
    }

    fn update_selected_addresses_scores(
        &self,
        selected_addresses: &mut ManagedVec<DelegationContractSelectionInfo<Self::Api>>,
        is_delegate: bool,
        total_stake: &BigUint,
        total_apy: u64,
        total_nodes: u64,
        min_egld: &BigUint,
        config: &ScoringConfig,
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
                config,
            );
            total_score += &score;
            let _ = selected_addresses.set(index, info);
        }

        total_score
    }

    fn calculate_max_providers(
        &self,
        amount_to_delegate: &BigUint<Self::Api>,
        min_egld: &BigUint<Self::Api>,
        providers_len: usize,
    ) -> usize {
        let amount_decimal =
            ManagedDecimal::<Self::Api, ConstDecimals<DECIMALS>>::from(amount_to_delegate.clone());
        let min_egld_decimal =
            ManagedDecimal::<Self::Api, ConstDecimals<DECIMALS>>::from(min_egld.clone());

        let max_providers_decimal = amount_decimal / min_egld_decimal;
        let max_providers_biguint = max_providers_decimal.trunc();

        let max_providers_limit = self.max_selected_providers().get();
        let max_providers = max_providers_biguint
            .clone()
            .min(max_providers_limit)
            .min(BigUint::from(providers_len as u64));

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

    fn calculate_share(&self, total_amount: &BigUint, cut_percentage: &BigUint) -> BigUint {
        total_amount * cut_percentage / BPS
    }

    fn add_delegation_address_in_list(&self, contract_address: ManagedAddress) {
        let mut delegation_addresses_mapper = self.delegation_addresses_list();

        delegation_addresses_mapper.insert(contract_address);
    }

    fn remove_delegation_address_from_list(&self, contract_address: &ManagedAddress) {
        self.delegation_addresses_list().remove(contract_address);
    }

    fn move_delegation_contract_to_back(&self, delegation_contract: &ManagedAddress) {
        self.remove_delegation_address_from_list(delegation_contract);

        self.delegation_addresses_list()
            .insert(delegation_contract.clone());
    }

    fn require_min_rounds_passed(&self) {
        // TODO: Implement once new hooks are available in the VM with the future mainnet upgrade
        return;
    }

    fn calculate_absolute_score(
        &self,
        config: &ScoringConfig,
        value: u64,
        min_value: u64,
        max_value: u64,
        exponential: bool,
        inverse: bool,
    ) -> BigUint {
        if value <= min_value {
            return if inverse {
                BigUint::from(config.max_score_per_category)
            } else {
                BigUint::zero()
            };
        }
        if value >= max_value {
            return if inverse {
                BigUint::zero()
            } else {
                BigUint::from(config.max_score_per_category)
            };
        }

        let position = value.saturating_sub(min_value);
        let range = max_value.saturating_sub(min_value);

        if exponential {
            let position = if inverse {
                range.saturating_sub(position)
            } else {
                position
            };
            let factor = BigUint::from(config.exponential_base)
                .pow((position * config.apy_growth_multiplier / range) as u32);
            (BigUint::from(config.max_score_per_category) * factor)
                / BigUint::from(config.exponential_base).pow(2u32)
        } else {
            if inverse {
                BigUint::from(position)
                    .mul(BigUint::from(config.max_score_per_category))
                    .div(range)
            } else {
                BigUint::from(range.saturating_sub(position))
                    .mul(BigUint::from(config.max_score_per_category))
                    .div(range)
            }
        }
    }

    fn calculate_relative_score(
        &self,
        config: &ScoringConfig,
        value: u64,
        total: u64,
        quadratic: bool,
        inverse: bool,
    ) -> BigUint {
        if total == 0 {
            return if inverse {
                BigUint::zero()
            } else {
                BigUint::from(config.max_score_per_category)
            };
        }

        let ratio = BigUint::from(value * BPS) / total;
        let base_ratio = if inverse {
            &BigUint::from(BPS) - &ratio
        } else {
            ratio
        };

        if quadratic {
            (BigUint::from(config.max_score_per_category) * base_ratio.pow(2))
                / BigUint::from(BPS).pow(2)
        } else {
            BigUint::from(config.max_score_per_category) * base_ratio / BPS
        }
    }

    fn get_scoring_config(&self) -> ScoringConfig {
        let map = self.scoring_config();
        require!(!map.is_empty(), ERROR_SCORING_CONFIG_NOT_SET);
        map.get()
    }
}
