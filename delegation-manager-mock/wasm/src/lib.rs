// Code generated by the multiversx-sc build system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                            1
// Async Callback (empty):               1
// Total number of exported functions:   3

#![no_std]

multiversx_sc_wasm_adapter::allocator!();
multiversx_sc_wasm_adapter::panic_handler!();

multiversx_sc_wasm_adapter::endpoints! {
    delegation_manager_mock
    (
        init => init
        claimMulti => claim_multiple
    )
}

multiversx_sc_wasm_adapter::async_callback_empty! {}
