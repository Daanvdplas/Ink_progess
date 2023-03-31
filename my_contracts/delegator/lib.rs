#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod caller {
    use ink::env::{
        call::{build_call, Call, ExecutionInput, Selector},
        CallFlags, DefaultEnvironment,
    };

    #[derive(Debug, Copy, Clone, PartialEq, Eq, scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
    )]
    pub enum Which {
        Adder,
        Subber,
    }

    #[ink(storage)]
    pub struct Delegator {
        /// Says which of `adder` or `subber` is currently in use.
        which: Which,
        /// The `accumulator` smart contract.
        acc_contract: AccountId,
        /// The `adder` smart contract.
        add_contract: AccountId,
        /// The `subber` smart contract.
        sub_contract: AccountId,
    }

    impl Delegator {
        #[ink(constructor)]
        pub fn new(
            acc_contract: AccountId,
            add_contract: AccountId,
            sub_contract: AccountId,
        ) -> Self {
            Delegator {
                which: Which::Adder,
                acc_contract,
                add_contract,
                sub_contract,
            }
        }

        #[ink(message)]
        pub fn get(&self) -> i32 {
            let method_selector = [0xC0, 0xDE, 0xCA, 0xF1];
            build_call::<DefaultEnvironment>()
                .call(self.acc_contract)
                .gas_limit(0)
                .transferred_value(0)
                .call_flags(CallFlags::default())
                .exec_input(ExecutionInput::new(method_selector.into()))
                .returns::<i32>()
                .try_invoke()
                .unwrap_or_else(|env_err| {
                    panic!(
                        "cross-contract call to {:?} failed due to {:?}",
                        self.acc_contract, env_err
                    )
                })
                .unwrap_or_else(|lang_err| {
                    panic!(
                        "cross-contract call to {:?} failed due to {:?}",
                        self.acc_contract, lang_err
                    )
                })
        }

        #[ink(message)]
        pub fn change(&self, by: i32) {
            let method_selector = [0xC0, 0xDE, 0xCA, 0xFE];
            let contract = match self.which {
                Which::Adder => self.add_contract,
                Which::Subber => self.sub_contract,
            };
            let _result = build_call::<DefaultEnvironment>()
                .call(contract)
                .call_flags(
                    CallFlags::default()
                        .set_tail_call(true)
                        .set_allow_reentry(true),
                )
                .exec_input(ExecutionInput::new(method_selector.into()).push_arg(by))
                .returns::<()>()
                .try_invoke();
        }

        #[ink(message)]
        pub fn switch(&mut self) {
            match self.which {
                Which::Adder => {
                    self.which = Which::Subber;
                }
                Which::Subber => {
                    self.which = Which::Adder;
                }
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[ink::test]
        fn new() {
            let delegator = Delegator::new([0x06; 32].into(), [0x07; 32].into(), [0x08; 32].into());
            assert_eq!(delegator.acc_contract, [0x06; 32].into());
            assert_eq!(delegator.add_contract, [0x07; 32].into());
            assert_eq!(delegator.sub_contract, [0x08; 32].into());
            assert_eq!(delegator.which, Which::Adder);
        }

        #[ink::test]
        fn switch() {
            let mut delegator =
                Delegator::new([0x06; 32].into(), [0x07; 32].into(), [0x08; 32].into());
            assert_eq!(delegator.which, Which::Adder);
            delegator.switch();
            assert_eq!(delegator.which, Which::Subber);
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;

        // #[ink_e2e::test]
        // async fn
    }
}
