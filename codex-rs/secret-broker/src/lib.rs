#![forbid(unsafe_code)]

//! Platform-containment contracts for Corbanu's isolated credential broker.

pub mod platform_contract;

#[cfg(test)]
#[path = "platform_contract_fixture_tests.rs"]
mod platform_contract_fixture_tests;
