#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod caller {
    use ink::env::{
        call::{build_call, Call, ExecutionInput, Selector},
        CallFlags, DefaultEnvironment,
    };

    #[ink(storage)]
    pub struct Caller {
        contract: AccountId,
    }

    impl Caller {
        #[ink(constructor)]
        pub fn new(contract: AccountId) -> Self {
            Caller { contract }
        }

        #[ink(message)]
        pub fn call(&self) {
            let method_selector = [0xC0, 0xDE, 0xCA, 0xFE];
            let _result = build_call::<<Self as ::ink::env::ContractEnv>::Env>()
                .call(self.contract)
                .call_flags(CallFlags::default())
                .exec_input(ExecutionInput::new(method_selector.into()))
                .returns::<()>()
                .try_invoke();
        }
    }
}
