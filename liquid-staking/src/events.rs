multiversx_sc::imports!();
multiversx_sc::derive_imports!();
use crate::contexts::base::StorageCache;

#[type_abi]
#[derive(TopEncode)]
pub struct ChangeLiquidityEvent<M: ManagedTypeApi> {
    caller: ManagedAddress<M>,
    ls_token_id: TokenIdentifier<M>,
    ls_token_supply: BigUint<M>,
    virtual_egld_reserve: BigUint<M>,
    fees_reserve: BigUint<M>,
    total_withdrawn_egld: BigUint<M>,
    pending_egld: BigUint<M>,
    pending_egld_for_unstake: BigUint<M>,
    pending_egld_for_unbond: BigUint<M>,
    block: u64,
    epoch: u64,
    timestamp: u64,
}

#[multiversx_sc::module]
pub trait EventsModule:
    crate::config::ConfigModule
    + crate::storage::StorageModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn emit_add_liquidity_event(
        &self,
        storage_cache: &StorageCache<Self>,
        egld_amount: &BigUint,
        external_caller: Option<ManagedAddress>,
    ) {
        let epoch = self.blockchain().get_block_epoch();
        let caller = match external_caller {
            Some(external_caller) => external_caller,
            None => self.blockchain().get_caller(),
        };
        self.add_liquidity_event(
            &egld_amount,
            &ChangeLiquidityEvent {
                caller: caller.clone(),
                ls_token_id: storage_cache.ls_token_id.clone(),
                ls_token_supply: storage_cache.ls_token_supply.clone(),
                virtual_egld_reserve: storage_cache.virtual_egld_reserve.clone(),
                fees_reserve: storage_cache.fees_reserve.clone(),
                total_withdrawn_egld: storage_cache.total_withdrawn_egld.clone(),
                pending_egld: storage_cache.pending_egld.clone(),
                pending_egld_for_unstake: storage_cache.pending_egld_for_unstake.clone(),
                pending_egld_for_unbond: storage_cache.pending_egld_for_unbond.clone(),
                block: self.blockchain().get_block_nonce(),
                epoch,
                timestamp: self.blockchain().get_block_timestamp(),
            },
        )
    }

    fn emit_remove_liquidity_event(&self, storage_cache: &StorageCache<Self>, ls_amount: &BigUint) {
        let epoch = self.blockchain().get_block_epoch();
        let caller = self.blockchain().get_caller();
        self.remove_liquidity_event(
            &ls_amount,
            &ChangeLiquidityEvent {
                caller: caller.clone(),
                ls_token_id: storage_cache.ls_token_id.clone(),
                ls_token_supply: storage_cache.ls_token_supply.clone(),
                virtual_egld_reserve: storage_cache.virtual_egld_reserve.clone(),
                fees_reserve: storage_cache.fees_reserve.clone(),
                total_withdrawn_egld: storage_cache.total_withdrawn_egld.clone(),
                pending_egld: storage_cache.pending_egld.clone(),
                pending_egld_for_unstake: storage_cache.pending_egld_for_unstake.clone(),
                pending_egld_for_unbond: storage_cache.pending_egld_for_unbond.clone(),
                block: self.blockchain().get_block_nonce(),
                epoch,
                timestamp: self.blockchain().get_block_timestamp(),
            },
        )
    }

    fn emit_claim_rewards_event(
        &self,
        storage_cache: &StorageCache<Self>,
        egld_amount: &BigUint,
        fees: &BigUint,
    ) {
        let epoch = self.blockchain().get_block_epoch();
        let caller = self.blockchain().get_caller();

        self.protocol_revenue_event(fees, epoch);
        self.claim_rewards_event(
            &egld_amount,
            &ChangeLiquidityEvent {
                caller: caller.clone(),
                ls_token_id: storage_cache.ls_token_id.clone(),
                ls_token_supply: storage_cache.ls_token_supply.clone(),
                virtual_egld_reserve: storage_cache.virtual_egld_reserve.clone(),
                fees_reserve: storage_cache.fees_reserve.clone(),
                total_withdrawn_egld: storage_cache.total_withdrawn_egld.clone(),
                pending_egld: storage_cache.pending_egld.clone(),
                pending_egld_for_unstake: storage_cache.pending_egld_for_unstake.clone(),
                pending_egld_for_unbond: storage_cache.pending_egld_for_unbond.clone(),
                block: self.blockchain().get_block_nonce(),
                epoch,
                timestamp: self.blockchain().get_block_timestamp(),
            },
        )
    }

    fn emit_withdraw_pending_event(
        &self,
        storage_cache: &StorageCache<Self>,
        egld_amount: &BigUint,
        delegation_contract: &ManagedAddress,
    ) {
        let epoch = self.blockchain().get_block_epoch();
        let caller = self.blockchain().get_caller();
        self.withdraw_pending_event(
            &egld_amount,
            &ChangeLiquidityEvent {
                caller: caller.clone(),
                ls_token_id: storage_cache.ls_token_id.clone(),
                ls_token_supply: storage_cache.ls_token_supply.clone(),
                virtual_egld_reserve: storage_cache.virtual_egld_reserve.clone(),
                fees_reserve: storage_cache.fees_reserve.clone(),
                total_withdrawn_egld: storage_cache.total_withdrawn_egld.clone(),
                pending_egld: storage_cache.pending_egld.clone(),
                pending_egld_for_unstake: storage_cache.pending_egld_for_unstake.clone(),
                pending_egld_for_unbond: storage_cache.pending_egld_for_unbond.clone(),
                block: self.blockchain().get_block_nonce(),
                epoch,
                timestamp: self.blockchain().get_block_timestamp(),
            },
            &delegation_contract,
        )
    }

    fn emit_general_liquidity_event(&self, storage_cache: &StorageCache<Self>) {
        let epoch = self.blockchain().get_block_epoch();
        let caller = self.blockchain().get_caller();
        self.general_liquidity_event(&ChangeLiquidityEvent {
            caller: caller.clone(),
            ls_token_id: storage_cache.ls_token_id.clone(),
            ls_token_supply: storage_cache.ls_token_supply.clone(),
            virtual_egld_reserve: storage_cache.virtual_egld_reserve.clone(),
            fees_reserve: storage_cache.fees_reserve.clone(),
            total_withdrawn_egld: storage_cache.total_withdrawn_egld.clone(),
            pending_egld: storage_cache.pending_egld.clone(),
            pending_egld_for_unstake: storage_cache.pending_egld_for_unstake.clone(),
            pending_egld_for_unbond: storage_cache.pending_egld_for_unbond.clone(),
            block: self.blockchain().get_block_nonce(),
            epoch,
            timestamp: self.blockchain().get_block_timestamp(),
        })
    }

    #[event("add_liquidity")]
    fn add_liquidity_event(
        &self,
        #[indexed] amount: &BigUint,
        #[indexed] change_liquidity_event: &ChangeLiquidityEvent<Self::Api>,
    );

    #[event("remove_liquidity")]
    fn remove_liquidity_event(
        &self,
        #[indexed] amount: &BigUint,
        #[indexed] change_liquidity_event: &ChangeLiquidityEvent<Self::Api>,
    );

    #[event("withdraw_pending")]
    fn withdraw_pending_event(
        &self,
        #[indexed] amount: &BigUint,
        #[indexed] change_liquidity_event: &ChangeLiquidityEvent<Self::Api>,
        #[indexed] delegation_contract: &ManagedAddress,
    );

    #[event("claim_rewards")]
    fn claim_rewards_event(
        &self,
        #[indexed] amount: &BigUint,
        #[indexed] change_liquidity_event: &ChangeLiquidityEvent<Self::Api>,
    );

    #[event("protocol_revenue")]
    fn protocol_revenue_event(&self, #[indexed] amount: &BigUint, #[indexed] epoch: u64);

    #[event("general_liquidity_event")]
    fn general_liquidity_event(
        &self,
        #[indexed] change_liquidity_event: &ChangeLiquidityEvent<Self::Api>,
    );
}
