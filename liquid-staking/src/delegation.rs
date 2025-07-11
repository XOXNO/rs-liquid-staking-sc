multiversx_sc::imports!();
use crate::{
    callback::{CallbackModule, CallbackProxy},
    errors::{
        ERROR_ALREADY_WHITELISTED, ERROR_DELEGATION_CAP, ERROR_NOT_WHITELISTED,
        ERROR_ONLY_DELEGATION_ADMIN,
    },
    proxy::proxy_delegation,
    structs::DelegationContractData,
    utils, ERROR_MAX_DELEGATION_ADDRESSES, ERROR_MAX_UN_DELEGATION_ADDRESSES,
    ERROR_MIN_EGLD_TO_DELEGATE, MIN_EGLD_TO_DELEGATE, MIN_GAS_FOR_ASYNC_CALL,
    MIN_GAS_FOR_WHITELIST_CALLBACK,
};

#[multiversx_sc::module]
pub trait DelegationModule:
    crate::config::ConfigModule
    + crate::storage::StorageModule
    + crate::events::EventsModule
    + crate::callback::CallbackModule
    + utils::generic::UtilsModule
    + crate::score::ScoreModule
    + crate::selection::SelectionModule
    + crate::liquidity_pool::LiquidityPoolModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[payable("EGLD")]
    #[endpoint(whitelistDelegationContract)]
    fn whitelist_delegation_contract(
        &self,
        contract_address: ManagedAddress,
        admin_address: ManagedAddress,
        total_staked: BigUint,
        delegation_contract_cap: BigUint,
        nr_nodes: u64,
        apy: u64,
    ) {
        let mapper = self.max_delegation_addresses();
        require!(
            self.delegation_addresses_list().len() <= mapper.get(),
            ERROR_MAX_DELEGATION_ADDRESSES
        );

        require!(
            self.un_delegation_addresses_list().len() <= mapper.get(),
            ERROR_MAX_UN_DELEGATION_ADDRESSES
        );

        let caller = self.blockchain().get_caller();
        self.is_manager(&caller, true);
        let map_delegation_contract_data = self.delegation_contract_data(&contract_address);
        require!(
            map_delegation_contract_data.is_empty(),
            ERROR_ALREADY_WHITELISTED
        );

        require!(
            delegation_contract_cap >= total_staked || delegation_contract_cap == BigUint::zero(),
            ERROR_DELEGATION_CAP
        );

        let payment = self.call_value().egld().clone_value();
        require!(
            payment >= MIN_EGLD_TO_DELEGATE,
            ERROR_MIN_EGLD_TO_DELEGATE
        );

        let contract_data = DelegationContractData {
            admin_address,
            total_staked,
            delegation_contract_cap,
            nr_nodes,
            apy,
            total_staked_from_ls_contract: BigUint::zero(),
            total_unstaked_from_ls_contract: BigUint::zero(),
            pending_staking_callback_amount: BigUint::zero(),
            pending_unstaking_callback_amount: BigUint::zero(),
            eligible: true,
        };

        map_delegation_contract_data.set(contract_data);

        self.tx()
            .to(&contract_address)
            .typed(proxy_delegation::DelegationMockProxy)
            .delegate()
            .egld(&payment)
            .gas(MIN_GAS_FOR_ASYNC_CALL)
            .callback(
                CallbackModule::callbacks(self).whitelist_delegation_contract_callback(
                    contract_address.clone(),
                    &payment,
                    &caller,
                ),
            )
            .gas_for_callback(MIN_GAS_FOR_WHITELIST_CALLBACK)
            .register_promise();
    }

    #[endpoint(changeDelegationContractAdmin)]
    fn change_delegation_contract_admin(
        &self,
        contract_address: ManagedAddress,
        admin_address: ManagedAddress,
    ) {
        let delegation_address_mapper = self.delegation_contract_data(&contract_address);
        require!(!delegation_address_mapper.is_empty(), ERROR_NOT_WHITELISTED);
        self.is_manager(&self.blockchain().get_caller(), true);
        delegation_address_mapper.update(|contract_data| {
            contract_data.admin_address = admin_address;
        });
    }

    #[endpoint(changeDelegationContractParams)]
    fn change_delegation_contract_params(
        &self,
        contract_address: ManagedAddress,
        total_staked: BigUint,
        delegation_contract_cap: BigUint,
        nr_nodes: u64,
        apy: u64,
        is_eligible: bool,
    ) {
        let caller = self.blockchain().get_caller();
        let delegation_address_mapper = self.delegation_contract_data(&contract_address);
        let old_contract_data = delegation_address_mapper.get();

        require!(!delegation_address_mapper.is_empty(), ERROR_NOT_WHITELISTED);

        require!(
            old_contract_data.admin_address == caller || self.is_manager(&caller, false),
            ERROR_ONLY_DELEGATION_ADMIN
        );

        require!(
            delegation_contract_cap >= total_staked || delegation_contract_cap == BigUint::zero(),
            ERROR_DELEGATION_CAP
        );

        delegation_address_mapper.update(|contract_data| {
            contract_data.total_staked = total_staked;
            contract_data.delegation_contract_cap = delegation_contract_cap;
            contract_data.nr_nodes = nr_nodes;
            contract_data.apy = apy;
            contract_data.eligible = is_eligible;
        });
    }
}
