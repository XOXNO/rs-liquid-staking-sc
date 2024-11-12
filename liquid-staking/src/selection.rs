multiversx_sc::imports!();
use crate::{
    structs::{
        DelegationContractInfo, DelegationContractSelectionInfo, DelegatorSelection, ScoringConfig,
    },
    StorageCache, DECIMALS, ERROR_BAD_DELEGATION_ADDRESS, ERROR_FAILED_TO_DISTRIBUTE,
    ERROR_NO_DELEGATION_CONTRACTS, ERROR_SCORING_CONFIG_NOT_SET, MIN_EGLD_TO_DELEGATE,
};

#[multiversx_sc::module]
pub trait SelectionModule:
    crate::storage::StorageModule + crate::config::ConfigModule + crate::score::ScoreModule
{
    #[inline]
    fn get_scoring_config(&self) -> ScoringConfig {
        let map = self.scoring_config();
        require!(!map.is_empty(), ERROR_SCORING_CONFIG_NOT_SET);
        map.get()
    }

    fn get_delegation_contract(
        &self,
        amount: &BigUint,
        is_delegate: bool,
        storage_cache: &mut StorageCache<Self>,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        let map_list = if is_delegate {
            self.delegation_addresses_list()
        } else {
            self.un_delegation_addresses_list()
        };

        require!(!map_list.is_empty(), ERROR_NO_DELEGATION_CONTRACTS);
        let min_egld = BigUint::from(MIN_EGLD_TO_DELEGATE);

        if !is_delegate {
            return self.handle_undelegation(&map_list, amount, &min_egld, storage_cache);
        }

        self.handle_delegation(&map_list, amount, &min_egld, storage_cache)
    }

    fn handle_delegation(
        &self,
        map_list: &SetMapper<Self::Api, ManagedAddress>,
        amount: &BigUint,
        min_egld: &BigUint,
        storage_cache: &mut StorageCache<Self>,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        let (mut selected_addresses, total_stake) =
            self.select_delegation_providers(map_list, amount, min_egld);

        require!(!selected_addresses.is_empty(), ERROR_BAD_DELEGATION_ADDRESS);

        let config = self.get_scoring_config();
        self.distribute_amount(
            &mut selected_addresses,
            amount,
            min_egld,
            true,
            &total_stake,
            &config,
            storage_cache,
        )
    }

    fn select_delegation_providers(
        &self,
        map_list: &SetMapper<Self::Api, ManagedAddress>,
        amount: &BigUint,
        min_egld: &BigUint,
    ) -> (
        ManagedVec<DelegationContractSelectionInfo<Self::Api>>,
        BigUint,
    ) {
        let max_providers = self.calculate_max_providers(amount, min_egld, map_list.len());
        let amount_per_provider = amount / &BigUint::from(max_providers as u64);
        let all_providers_limit =
            (amount / &BigUint::from(map_list.len() as u64)).max(min_egld.clone());

        let mut selected_addresses = ManagedVec::new();
        let mut total_stake = BigUint::zero();

        for address in map_list.iter() {
            let contract_data = self.delegation_contract_data(&address).get();

            if self.is_delegation_provider_eligible(
                &contract_data,
                &amount_per_provider,
                &all_providers_limit,
                min_egld,
            ) {
                total_stake += &contract_data.total_staked_from_ls_contract;
                selected_addresses.push(self.create_selection_info(&address, &contract_data));
            }

            if selected_addresses.len() == max_providers {
                break;
            }
        }

        (selected_addresses, total_stake)
    }

    fn handle_undelegation(
        &self,
        map_list: &SetMapper<Self::Api, ManagedAddress>,
        amount: &BigUint,
        min_egld: &BigUint,
        storage_cache: &mut StorageCache<Self>,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        let (mut selected_providers, total_stake) =
            self.select_undelegation_providers(map_list, amount, min_egld);

        require!(!selected_providers.is_empty(), ERROR_BAD_DELEGATION_ADDRESS);

        let config = self.get_scoring_config();
        self.distribute_amount(
            &mut selected_providers,
            amount,
            min_egld,
            false,
            &total_stake,
            &config,
            storage_cache,
        )
    }

    fn select_undelegation_providers(
        &self,
        map_list: &SetMapper<Self::Api, ManagedAddress>,
        amount: &BigUint,
        min_egld: &BigUint,
    ) -> (
        ManagedVec<DelegationContractSelectionInfo<Self::Api>>,
        BigUint,
    ) {
        let mut selected_providers = ManagedVec::new();
        let mut total_stake = BigUint::zero();
        let mut remaining = amount.clone();
        let max_providers = self.max_selected_providers().get().to_u64().unwrap() as usize;

        for address in map_list.iter() {
            // Check both max providers and remaining amount
            if remaining == BigUint::zero() || selected_providers.len() >= max_providers {
                break;
            }

            let contract_data = self.delegation_contract_data(&address).get();
            let staked = &contract_data.total_staked_from_ls_contract;

            if staked >= &remaining || staked >= min_egld {
                let amount_to_take = if staked > &(min_egld.clone() * 2u64) {
                    staked - min_egld // Leave min_egld to avoid dust
                } else {
                    staked.clone() // Take all if small amount
                };

                if amount_to_take > BigUint::zero() {
                    total_stake += staked;
                    selected_providers.push(self.create_selection_info(&address, &contract_data));

                    if staked >= &remaining {
                        break;
                    }
                    remaining -= amount_to_take;
                }
            }
        }

        (selected_providers, total_stake)
    }

    fn is_delegation_provider_eligible(
        &self,
        contract_data: &DelegationContractInfo<Self::Api>,
        amount_per_provider: &BigUint,
        all_providers_limit: &BigUint,
        min_egld: &BigUint,
    ) -> bool {
        if !contract_data.eligible {
            return false;
        }

        contract_data.delegation_contract_cap == BigUint::zero()
            || &contract_data.delegation_contract_cap - &contract_data.total_staked
                >= *amount_per_provider
            || &contract_data.delegation_contract_cap - &contract_data.total_staked
                >= *all_providers_limit
            || &contract_data.delegation_contract_cap - &contract_data.total_staked >= *min_egld
    }

    fn distribute_amount(
        &self,
        selected_addresses: &mut ManagedVec<DelegationContractSelectionInfo<Self::Api>>,
        amount: &BigUint,
        min_egld: &BigUint,
        is_delegate: bool,
        total_stake: &BigUint,
        config: &ScoringConfig,
        storage_cache: &mut StorageCache<Self>,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        let mut result = ManagedVec::new();
        let mut remaining_amount = amount.clone();

        // Calculate scores
        let total_score = self.update_selected_addresses_scores(
            selected_addresses,
            is_delegate,
            total_stake,
            config,
        );

        // Distribute based on scores
        for info in selected_addresses.iter() {
            if remaining_amount < *min_egld {
                break;
            }

            let amount_to_delegate = self.calculate_provider_amount(
                &info,
                amount,
                &remaining_amount,
                &total_score,
                is_delegate,
            );

            if amount_to_delegate >= *min_egld {
                result.push(DelegatorSelection::new(
                    info.address.clone(),
                    amount_to_delegate.clone(),
                    if is_delegate {
                        info.space_left.clone()
                    } else {
                        Some(info.total_staked_from_ls_contract.clone())
                    },
                ));
                remaining_amount -= amount_to_delegate;
            }
        }

        self.handle_remaining_amount(
            &mut result,
            remaining_amount,
            min_egld,
            is_delegate,
            storage_cache,
        );

        result
    }

    fn calculate_provider_amount(
        &self,
        info: &DelegationContractSelectionInfo<Self::Api>,
        total_amount: &BigUint,
        remaining_amount: &BigUint,
        total_score: &BigUint,
        is_delegate: bool,
    ) -> BigUint {
        let proportion = if total_score > &BigUint::zero() {
            (&info.score * total_amount) / total_score
        } else {
            remaining_amount.clone()
        };

        if is_delegate {
            match &info.space_left {
                Some(space_left) => proportion.min(space_left.clone()),
                None => proportion,
            }
        } else {
            proportion.min(info.total_staked_from_ls_contract.clone())
        }
    }

    fn handle_remaining_amount(
        &self,
        result: &mut ManagedVec<DelegatorSelection<Self::Api>>,
        remaining_amount: BigUint,
        min_egld: &BigUint,
        is_delegate: bool,
        storage_cache: &mut StorageCache<Self>,
    ) {
        if remaining_amount > BigUint::zero() {
            if remaining_amount >= *min_egld {
                if is_delegate {
                    storage_cache.pending_egld += remaining_amount;
                } else {
                    storage_cache.pending_egld_for_unstake += remaining_amount;
                }
            } else {
                let distributed =
                    self.try_distribute_dust(result, &remaining_amount, min_egld, is_delegate);
                require!(distributed, ERROR_FAILED_TO_DISTRIBUTE);
            }
        }
    }

    fn try_distribute_dust(
        &self,
        result: &mut ManagedVec<DelegatorSelection<Self::Api>>,
        remaining_amount: &BigUint,
        min_egld: &BigUint,
        is_delegate: bool,
    ) -> bool {
        for i in 0..result.len() {
            let selection = result.get(i);

            if self.can_add_remaining_to_provider(
                &selection,
                remaining_amount,
                min_egld,
                is_delegate,
            ) {
                self.update_provider_amount(result, i, &selection, remaining_amount);
                return true;
            }
        }
        false
    }

    fn can_add_remaining_to_provider(
        &self,
        selection: &DelegatorSelection<Self::Api>,
        remaining_amount: &BigUint,
        min_egld: &BigUint,
        is_delegate: bool,
    ) -> bool {
        let new_amount = selection.amount.clone() + remaining_amount;

        if is_delegate {
            selection.space_left.is_none()
                || (selection.space_left.is_some()
                    && new_amount <= selection.space_left.clone().unwrap())
        } else {
            // For undelegation, we can use the space_left field which contains total_staked_from_ls_contract
            let current_staked = selection.space_left.clone().unwrap_or_default();
            &new_amount <= &current_staked
                && (&current_staked - &new_amount == BigUint::zero()
                    || &current_staked - &new_amount >= *min_egld)
        }
    }

    fn update_provider_amount(
        &self,
        result: &mut ManagedVec<DelegatorSelection<Self::Api>>,
        index: usize,
        selection: &DelegatorSelection<Self::Api>,
        remaining_amount: &BigUint,
    ) {
        let new_amount = selection.amount.clone() + remaining_amount;
        let _ = result.set(
            index,
            DelegatorSelection::new(
                selection.delegation_address.clone(),
                new_amount,
                selection.space_left.clone(),
            ),
        );
    }

    fn create_selection_info(
        &self,
        address: &ManagedAddress,
        contract_data: &DelegationContractInfo<Self::Api>,
    ) -> DelegationContractSelectionInfo<Self::Api> {
        DelegationContractSelectionInfo {
            address: address.clone(),
            space_left: if contract_data.delegation_contract_cap == BigUint::zero() {
                None
            } else {
                Some(&contract_data.delegation_contract_cap - &contract_data.total_staked)
            },
            total_staked: contract_data.total_staked.clone(),
            apy: contract_data.apy,
            score: BigUint::zero(),
            nr_nodes: contract_data.nr_nodes,
            total_staked_from_ls_contract: contract_data.total_staked_from_ls_contract.clone(),
        }
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
}
