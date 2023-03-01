#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
pub mod accumulator {
    use ink::env::{
        call::{build_call, Call, ExecutionInput, Selector},
        CallFlags, DefaultEnvironment,
    };
    /// Holds a simple `i32` value that can be incremented and decremented.
    #[ink(storage)]
    pub struct Accumulator {
        value: i32,
    }

    impl Accumulator {
        /// Initializes the value to the initial value.
        #[ink(constructor)]
        pub fn new(init_value: i32) -> Self {
            Self { value: init_value }
        }

        /// Mutates the internal value.
        #[ink(message, selector = 0xC0DECAFE)]
        pub fn inc(&mut self, by: i32) {
            self.value += by;
        }

        /// Returns the current state.
        #[ink(message, selector = 0xC0DECAF1)]
        pub fn get(&self) -> i32 {
            self.value
        }
    }
}
