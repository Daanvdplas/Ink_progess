#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod subber {
    use ink::env::{
        call::{build_call, Call, ExecutionInput, Selector},
        CallFlags, DefaultEnvironment,
    };
    /// Decreases the underlying `accumulator` value.
    #[ink(storage)]
    pub struct Subber {
        /// The `accumulator` to store the value.
        sub_contract: AccountId,
    }

    impl Subber {
        /// Creates a new `subber` from the given `accumulator`.
        #[ink(constructor)]
        pub fn new(sub_contract: AccountId) -> Self {
            Self { sub_contract }
        }

        /// Decreases the `accumulator` value by some amount.
        #[ink(message, selector = 0xC0DECAFE)]
        pub fn dec(&mut self, by: i32) {
            let method_selector = [0xC0, 0xDE, 0xCA, 0xFE];
            let _result = build_call::<<Self as ::ink::env::ContractEnv>::Env>()
                .call(self.sub_contract)
                .call_flags(CallFlags::default())
                .exec_input(ExecutionInput::new(method_selector.into()).push_arg(-by))
                .returns::<()>()
                .try_invoke();
        }
    }
}
