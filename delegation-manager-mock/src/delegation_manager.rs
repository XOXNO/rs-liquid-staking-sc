#![no_std]

multiversx_sc::imports!();
pub mod delegation_proxy;
#[multiversx_sc::derive::contract]
pub trait DelegationManagerMock {
    #[init]
    fn init(&self) {}

    #[endpoint(claimMulti)]
    fn claim_multiple(&self, addresses: MultiValueEncoded<ManagedAddress>) -> BigUint {
        let mut total_rewards = BigUint::zero();
        let caller = self.blockchain().get_caller();
        for address in addresses {
            let back_transfers = self
                .tx()
                .to(&address)
                .typed(delegation_proxy::DelegationMockProxy)
                .claim_rewards()
                .returns(ReturnsBackTransfers)
                .sync_call();

            total_rewards += back_transfers.total_egld_amount;
        }
        self.tx().to(&caller).egld(&total_rewards).transfer();
        total_rewards
    }
}