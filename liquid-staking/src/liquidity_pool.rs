multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::contexts::base::StorageCache;
use crate::structs::UnstakeTokenAttributes;
use crate::{errors::*, storage};

use super::config;

pub const UNDELEGATE_TOKEN_URI: &[u8] =
    b"https://ipfs.io/ipfs/QmTtBCeg5zLz2fnZedfukavPsfYhB7A95EXUkEddcNwNDX";

#[multiversx_sc::module]
pub trait LiquidityPoolModule: config::ConfigModule + storage::StorageModule {
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
                + &storage_cache.rewards_reserve
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
            let uri = ManagedBuffer::from(UNDELEGATE_TOKEN_URI);
            let token_id = self.unstake_token().get_token_id();

            // Always add extra one to the initial MetaESDT amount
            // The extra 1 will remain in the contract and will be used to add later quantities for the same epoch
            let new_nonce = self.send().esdt_nft_create(
                &token_id,
                &amount.add(&BigUint::from(1u64)),
                &sc_format!("Release epoch #{}", epoch),
                &BigUint::zero(),
                &ManagedBuffer::new(),
                attributes,
                &ManagedVec::from_single_item(uri),
            );

            nonce.set(new_nonce);

            if new_nonce > 1 {
                self.clean_old_unbond_epochs(new_nonce - 1);
            }

            EsdtTokenPayment::new(token_id, new_nonce, amount.clone())
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

    fn clean_old_unbond_epochs(&self, nonce: u64) {
        let epoch = self.blockchain().get_block_epoch();
        let map_token = self.unstake_token();

        let balance = map_token.get_balance(nonce);

        if balance == BigUint::zero() {
            return;
        }

        let attributes: UnstakeTokenAttributes = map_token.get_token_attributes(nonce);
        if attributes.unstake_epoch < epoch {
            self.unstake_token_nonce(attributes.unbond_epoch).clear();
            // The protocol always holds 1 unit of the MetaESDT token in the contract
            let balance = map_token.get_balance(nonce);
            map_token.nft_burn(nonce, &balance);
        }
    }
}
