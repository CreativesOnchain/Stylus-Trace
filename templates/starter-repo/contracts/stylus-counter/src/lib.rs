#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

use stylus_sdk::alloy_primitives::U256;
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct Counter {
        uint256 value;
    }
}

#[public]
impl Counter {
    pub fn increment(&mut self, amount: U256) -> U256 {
        let next = self.value.get() + amount;
        self.value.set(next);
        next
    }

    pub fn get(&self) -> U256 {
        self.value.get()
    }
}
