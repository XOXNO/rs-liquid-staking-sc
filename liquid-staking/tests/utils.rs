use std::ops::Mul;

use multiversx_sc::types::BigUint;
use multiversx_sc_scenario::{api::StaticApi, num_bigint, DebugApi};

pub fn bytes_to_str(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).unwrap()
}

pub fn exp9(value: u64) -> BigUint<StaticApi> {
    BigUint::from(value).mul(BigUint::from(10u64).pow(9))
}

pub fn exp15(value: u64) -> BigUint<StaticApi> {
    BigUint::from(value).mul(BigUint::from(10u64).pow(15))
}

pub fn exp17(value: u64) -> BigUint<StaticApi> {
    BigUint::from(value).mul(BigUint::from(10u64).pow(17))
}

pub fn exp18(value: u64) -> BigUint<StaticApi> {
    BigUint::from(value).mul(BigUint::from(10u64).pow(18))
}

pub fn exp18_128(value: u64) -> u128 {
    u128::from(value).mul(10u128.pow(18))
}

pub fn exp(value: u128) -> BigUint<StaticApi> {
    BigUint::from(value)
}

pub fn to_managed_biguint(value: num_bigint::BigUint) -> BigUint<DebugApi> {
    BigUint::from_bytes_be(&value.to_bytes_be())
}
