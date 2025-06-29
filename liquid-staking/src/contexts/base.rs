multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::State;

pub struct StorageCache<'a, C>
where
    C: crate::config::ConfigModule,
{
    sc_ref: &'a C,
    pub contract_state: State,
    pub ls_token_id: TokenIdentifier<C::Api>,
    pub ls_token_supply: BigUint<C::Api>,
    pub virtual_egld_reserve: BigUint<C::Api>,
    pub fees_reserve: BigUint<C::Api>,
    pub total_withdrawn_egld: BigUint<C::Api>,
    pub pending_egld: BigUint<C::Api>,
    pub pending_egld_for_unstake: BigUint<C::Api>,
    pub pending_egld_for_unbond: BigUint<C::Api>,
}

impl<'a, C> StorageCache<'a, C>
where
    C: crate::config::ConfigModule,
{
    pub fn new(sc_ref: &'a C) -> Self {
        StorageCache {
            contract_state: sc_ref.state().get(),
            ls_token_id: sc_ref.ls_token().get_token_id(),
            ls_token_supply: sc_ref.ls_token_supply().get(),
            virtual_egld_reserve: sc_ref.virtual_egld_reserve().get(),
            fees_reserve: sc_ref.fees_reserve().get(),
            total_withdrawn_egld: sc_ref.total_withdrawn_egld().get(),
            pending_egld: sc_ref.pending_egld().get(),
            pending_egld_for_unstake: sc_ref.pending_egld_for_unstake().get(),
            pending_egld_for_unbond: sc_ref.pending_egld_for_unbond().get(),
            sc_ref,
        }
    }
}

impl<C> Drop for StorageCache<'_, C>
where
    C: crate::config::ConfigModule,
{
    fn drop(&mut self) {
        // commit changes to storage for the mutable fields
        self.sc_ref.ls_token_supply().set(&self.ls_token_supply);
        self.sc_ref
            .virtual_egld_reserve()
            .set(&self.virtual_egld_reserve);
        self.sc_ref.fees_reserve().set(&self.fees_reserve);
        self.sc_ref
            .total_withdrawn_egld()
            .set(&self.total_withdrawn_egld);
        self.sc_ref.pending_egld().set(&self.pending_egld);
        self.sc_ref
            .pending_egld_for_unstake()
            .set(&self.pending_egld_for_unstake);
        self.sc_ref
            .pending_egld_for_unbond()
            .set(&self.pending_egld_for_unbond);
    }
}
