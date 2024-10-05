multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::config::UNBOND_PERIOD;
use crate::contexts::base::StorageCache;
use crate::errors::*;
use crate::structs::UnstakeTokenAttributes;

use super::config;

const MINIMUM_LIQUIDITY: u64 = 1_000;

#[multiversx_sc::module]
pub trait LiquidityPoolModule: config::ConfigModule {
    fn pool_add_liquidity(
        &self,
        token_amount: &BigUint,
        storage_cache: &mut StorageCache<Self>,
    ) -> BigUint {
        let ls_amount = self.get_ls_amount(token_amount, storage_cache);

        storage_cache.ls_token_supply += &ls_amount;
        storage_cache.virtual_egld_reserve += token_amount;

        ls_amount
    }

    fn pool_remove_liquidity(
        &self,
        token_amount: &BigUint,
        storage_cache: &mut StorageCache<Self>,
    ) -> BigUint {
        let egld_amount = self.get_egld_amount(token_amount, storage_cache);
        storage_cache.ls_token_supply -= token_amount;
        storage_cache.virtual_egld_reserve -= &egld_amount;

        egld_amount
    }

    fn get_egld_amount(
        &self,
        ls_token_amount: &BigUint,
        storage_cache: &StorageCache<Self>,
    ) -> BigUint {
        require!(
            storage_cache.ls_token_supply >= ls_token_amount + MINIMUM_LIQUIDITY,
            ERROR_NOT_ENOUGH_LP
        );

        let egld_amount =
            ls_token_amount * &storage_cache.virtual_egld_reserve / &storage_cache.ls_token_supply;
        require!(egld_amount > 0u64, ERROR_INSUFFICIENT_LIQ_BURNED);

        egld_amount
    }

    fn get_ls_amount(
        &self,
        token_amount: &BigUint,
        storage_cache: &mut StorageCache<Self>,
    ) -> BigUint {
        let ls_amount = if storage_cache.virtual_egld_reserve > 0 {
            token_amount.clone() * &storage_cache.ls_token_supply
                / (&storage_cache.virtual_egld_reserve + &storage_cache.rewards_reserve)
        } else {
            token_amount.clone()
        };

        require!(ls_amount > 0, ERROR_INSUFFICIENT_LIQUIDITY);

        ls_amount
    }

    fn mint_ls_token(&self, amount: BigUint) -> EsdtTokenPayment<Self::Api> {
        self.ls_token().mint(amount)
    }

    fn burn_ls_token(&self, amount: &BigUint) {
        self.ls_token().burn(amount);
    }

    fn mint_unstake_tokens<T: TopEncode>(
        &self,
        attributes: &T,
        amount: &BigUint,
        epoch: u64,
    ) -> EsdtTokenPayment<Self::Api> {
        let nonce = self.unstake_token_nonce(epoch);
        if nonce.is_empty() {
            let payment = self.unstake_token().nft_create_named(
                amount.clone(),
                &sc_format!("Release epoch #{}", epoch),
                &attributes,
            );
            nonce.set(payment.token_nonce);
            payment
        } else {
            self.unstake_token()
                .nft_add_quantity(nonce.get(), amount.clone())
        }
    }

    fn burn_unstake_tokens(&self, token_nonce: u64, amount: &BigUint) {
        self.unstake_token().nft_burn(token_nonce, amount);
    }

    fn undelegate_amount(&self, egld_to_unstake: &BigUint, caller: &ManagedAddress) {
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_epoch = current_epoch + UNBOND_PERIOD;

        let virtual_position = UnstakeTokenAttributes {
            unstake_epoch: current_epoch,
            unbond_epoch,
        };

        let user_payment =
            self.mint_unstake_tokens(&virtual_position, egld_to_unstake, unbond_epoch);

        self.tx()
            .to(caller)
            .single_esdt(
                &user_payment.token_identifier,
                user_payment.token_nonce,
                &user_payment.amount,
            )
            .transfer();
    }
}
