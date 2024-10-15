multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::contexts::base::StorageCache;
use crate::errors::*;

use super::config;

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
            &storage_cache.ls_token_supply >= ls_token_amount,
            ERROR_NOT_ENOUGH_LP
        );

        let egld_amount =
            ls_token_amount * &storage_cache.virtual_egld_reserve / &storage_cache.ls_token_supply;

        require!(egld_amount > BigUint::zero(), ERROR_INSUFFICIENT_LIQ_BURNED);

        egld_amount
    }

    fn get_ls_amount(&self, token_amount: &BigUint, storage_cache: &StorageCache<Self>) -> BigUint {
        let ls_amount = if storage_cache.virtual_egld_reserve > BigUint::zero() {
            token_amount.clone() * &storage_cache.ls_token_supply
                / &storage_cache.virtual_egld_reserve
        } else {
            token_amount.clone()
        };

        require!(ls_amount > BigUint::zero(), ERROR_INSUFFICIENT_LIQUIDITY);

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
                attributes,
            );

            // Always add one to the initial MetaESDT so we can add later quantities for the same epoch
            self.unstake_token()
                .nft_add_quantity(payment.token_nonce, BigUint::from(1u64));

            nonce.set(payment.token_nonce);
            payment
        } else {
            let payment = self
                .unstake_token()
                .nft_add_quantity(nonce.get(), amount.clone());
            payment
        }
    }

    fn burn_unstake_tokens(&self, token_nonce: u64, amount: &BigUint) {
        self.unstake_token().nft_burn(token_nonce, amount);
    }
}
