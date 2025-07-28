multiversx_sc::imports!();
use crate::{
    constants::{BPS, ROUNDS_LEFT_TO_END_EPOCH, ROUNDS_PER_EPOCH},
    errors::ERROR_ROUNDS_NOT_PASSED,
    structs::{DelegatorSelection, State},
    StorageCache, ERROR_NOT_ACTIVE, MIN_EGLD_TO_DELEGATE,
};

#[multiversx_sc::module]
pub trait UtilsModule:
    crate::storage::StorageModule
    + crate::config::ConfigModule
    + crate::score::ScoreModule
    + crate::selection::SelectionModule
{
    fn get_contracts_for_delegate(
        &self,
        amount_to_delegate: &BigUint,
        storage_cache: &mut StorageCache<Self>,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        self.get_delegation_contract(amount_to_delegate, true, storage_cache)
    }

    fn get_contracts_for_undelegate(
        &self,
        amount_to_undelegate: &BigUint,
        storage_cache: &mut StorageCache<Self>,
    ) -> ManagedVec<DelegatorSelection<Self::Api>> {
        self.get_delegation_contract(amount_to_undelegate, false, storage_cache)
    }

    fn calculate_share(&self, total_amount: &BigUint, cut_percentage: &BigUint) -> BigUint {
        total_amount * cut_percentage / BPS
    }

    fn add_delegation_address_in_list(&self, contract_address: ManagedAddress) {
        let mut delegation_addresses_mapper = self.delegation_addresses_list();

        delegation_addresses_mapper.insert(contract_address);
    }

    fn add_un_delegation_address_in_list(&self, contract_address: ManagedAddress) {
        let mut un_delegation_addresses_mapper = self.un_delegation_addresses_list();

        un_delegation_addresses_mapper.insert(contract_address);
    }

    fn remove_delegation_address_from_list(&self, contract_address: &ManagedAddress) {
        self.delegation_addresses_list().remove(contract_address);
    }

    fn remove_un_delegation_address_from_list(&self, contract_address: &ManagedAddress) {
        self.un_delegation_addresses_list().remove(contract_address);
    }

    fn move_delegation_contract_to_back(&self, delegation_contract: &ManagedAddress) {
        self.remove_delegation_address_from_list(delegation_contract);

        self.delegation_addresses_list()
            .insert(delegation_contract.clone());
    }

    fn move_un_delegation_contract_to_back(&self, un_delegation_contract: &ManagedAddress) {
        self.remove_un_delegation_address_from_list(un_delegation_contract);

        self.un_delegation_addresses_list()
            .insert(un_delegation_contract.clone());
    }

    #[inline]
    fn is_state_active(&self, state: State) {
        require!(State::Active == state, ERROR_NOT_ACTIVE);
    }

    // Swap amount between pending and payment for both delegation and undelegation
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

    fn get_action_amount(
        &self,
        pending_amount: &BigUint,
        payment_amount: &BigUint,
    ) -> (BigUint, BigUint) {
        let min_egld_amount = BigUint::from(MIN_EGLD_TO_DELEGATE);

        if self.can_perform_instant_action(pending_amount, payment_amount, &min_egld_amount) {
            // Case 1: Full instant swap
            (payment_amount.clone(), BigUint::zero())
        } else if self.can_perform_fully_redeem_pending_amount(
            pending_amount,
            payment_amount,
            &min_egld_amount,
        ) {
            let egld_to_add_liquidity = payment_amount - pending_amount;
            (pending_amount.clone(), egld_to_add_liquidity)
        } else {
            self.calculate_partial_redeem_amount(pending_amount, payment_amount, &min_egld_amount)
        }
    }

    fn can_perform_instant_action(
        &self,
        pending_amount: &BigUint,
        payment_amount: &BigUint,
        min_egld_amount: &BigUint,
    ) -> bool {
        pending_amount == payment_amount
            || (pending_amount >= payment_amount
                && (pending_amount - payment_amount) >= *min_egld_amount)
    }

    fn can_perform_fully_redeem_pending_amount(
        &self,
        pending_amount: &BigUint,
        payment_amount: &BigUint,
        min_egld_amount: &BigUint,
    ) -> bool {
        payment_amount > pending_amount && (payment_amount - pending_amount) >= *min_egld_amount
    }

    fn calculate_partial_redeem_amount(
        &self,
        pending_amount: &BigUint,
        payment_amount: &BigUint,
        min_egld_amount: &BigUint,
    ) -> (BigUint, BigUint) {
        let possible_instant_amount =
            self.calculate_instant_amount(payment_amount, pending_amount, min_egld_amount);

        if possible_instant_amount > BigUint::zero()
            && pending_amount >= &possible_instant_amount
            && (pending_amount - &possible_instant_amount) >= *min_egld_amount
        {
            let left_over_amount = payment_amount - &possible_instant_amount;
            (possible_instant_amount, left_over_amount)
        } else {
            // Fallback: full amount action
            (BigUint::zero(), payment_amount.clone())
        }
    }

    // Function to check if enough rounds have passed since the start of the epoch
    // This is used to check if the contract is in the last few rounds of the epoch to allow pending actions for delegation and undelegation
    fn require_rounds_passed(&self) {
        let current_round = self.blockchain().get_block_round();
        let start_round = self.blockchain().epoch_start_block_round();

        let end_round = start_round + ROUNDS_PER_EPOCH;
        let rounds_passed = end_round - current_round;

        require!(
            rounds_passed <= ROUNDS_LEFT_TO_END_EPOCH,
            ERROR_ROUNDS_NOT_PASSED
        );
    }
}
