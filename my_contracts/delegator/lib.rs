#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod caller {
    use accumulator::AccumulatorRef;
    use adder::AdderRef;
    use ink::env::{
        call::{build_call, ExecutionInput},
        CallFlags, DefaultEnvironment,
    };
    use subber::SubberRef;

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
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(additional_contracts = "accumulator/Cargo.toml")]
        async fn instantiate_accumulator(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate `accumulator` contract
            let init_value = 10;
            let acc_constructor = AccumulatorRef::new(init_value);
            let acc_contract_account_id = client
                .instantiate("accumulator", &ink_e2e::alice(), acc_constructor, 0, None)
                .await
                .expect("accumulator contract instantiation failed")
                .account_id;

            // Build `get` message of `accumulator` contract and execute
            let get_message =
                ink_e2e::build_message::<AccumulatorRef>(acc_contract_account_id.clone())
                    .call(|accumulator| accumulator.get());
            let get_result = client
                .call_dry_run(&ink_e2e::eve(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value);
            Ok(())
        }

        #[ink_e2e::test(additional_contracts = "adder/Cargo.toml")]
        async fn instantiate_adder(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Instantiate `accumulator` contract
            let init_value = 10;
            let acc_constructor = AccumulatorRef::new(init_value);
            let acc_contract_account_id = client
                .instantiate("accumulator", &ink_e2e::alice(), acc_constructor, 0, None)
                .await
                .expect("accumulator contract instantiation failed")
                .account_id;

            // Build `get` message of `accumulator` contract and execute
            let get_message =
                ink_e2e::build_message::<AccumulatorRef>(acc_contract_account_id.clone())
                    .call(|accumulator| accumulator.get());
            let get_result = client
                .call_dry_run(&ink_e2e::eve(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value);

            // Instantiate `adder` contract
            let adder_constructor = AdderRef::new(acc_contract_account_id);
            let adder_contract_account_id = client
                .instantiate("adder", &ink_e2e::alice(), adder_constructor, 0, None)
                .await
                .expect("adder contract instantiation failed")
                .account_id;
            Ok(())
        }
        // #[ink_e2e::test(additional_contracts = "accumulator/Cargo.toml subber/Cargo.toml")]
        // #[ink_e2e::test(additional_contracts = "subber/Cargo.toml")]
        // async fn decrease(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        // Instantiate `accumulator` contract
        // let init_value = 10;
        // let acc_constructor = AccumulatorRef::new(init_value);
        // let acc_contract_account_id = client
        //     .instantiate("accumulator", &ink_e2e::alice(), acc_constructor, 0, None)
        //     .await
        //     .expect("accumulator contract instantiation failed")
        //     .account_id;

        // Build `get` message of `accumulator` contract and execute
        // let get_message =
        //     ink_e2e::build_message::<AccumulatorRef>(acc_contract_account_id.clone())
        //         .call(|accumulator| accumulator.get());
        // let get_result = client
        //     .call_dry_run(&ink_e2e::eve(), &get_message, 0, None)
        //     .await;
        // assert_eq!(get_result.return_value(), init_value);

        // Instantiate `subber` contract
        // let subber_constructor = SubberRef::new(acc_contract_account_id);
        // let subber_constructor = SubberRef::new([9; 32].into());
        // println!("subber_constructor");
        // let subber_contract_account_id = client
        //     .instantiate("subber", &ink_e2e::alice(), subber_constructor, 0, None)
        //     .await
        //     .expect("subber contract instantiation failed")
        //     .account_id;

        // Build `decrease` message of `subber` contract and execute
        // let decrease = 10;
        // let dec_message =
        //     ink_e2e::build_message::<SubberRef>(subber_contract_account_id.clone())
        //         .call(|subber| subber.dec(decrease));
        // let inc_result = client.call(&ink_e2e::eve(), dec_message, 0, None).await;
        // assert!(inc_result.is_ok());

        // Execute `get` message of `accumulator` contract
        // let get_result = client
        //     .call_dry_run(&ink_e2e::eve(), &get_message, 0, None)
        //     .await;
        // assert_eq!(get_result.return_value(), init_value - decrease);
        // Ok(())
        // }

        // #[ink_e2e::test(additional_contracts = "accumulator/Cargo.toml adder/Cargo.toml")]
        // async fn increase(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        //     // Instantiate `accumulator` contract
        //     let init_value = 10;
        //     let acc_constructor = AccumulatorRef::new(init_value);
        //     let acc_contract_account_id = client
        //         .instantiate("accumulator", &ink_e2e::alice(), acc_constructor, 0, None)
        //         .await
        //         .expect("accumulator contract instantiation failed")
        //         .account_id;

        //     // Build `get` message of `accumulator` contract and execute
        //     let get_message =
        //         ink_e2e::build_message::<AccumulatorRef>(acc_contract_account_id.clone())
        //             .call(|accumulator| accumulator.get());
        //     let get_result = client
        //         .call_dry_run(&ink_e2e::eve(), &get_message, 0, None)
        //         .await;
        //     assert_eq!(get_result.return_value(), init_value);

        //     // Instantiate `adder` contract
        //     let adder_constructor = AdderRef::new(acc_contract_account_id);
        //     let adder_contract_account_id = client
        //         .instantiate("adder", &ink_e2e::alice(), adder_constructor, 0, None)
        //         .await
        //         .expect("adder contract instantiation failed")
        //         .account_id;

        //     // Build `increase` message of `adder` contract and execute
        //     let increase = 10;
        //     let inc_message = ink_e2e::build_message::<AdderRef>(adder_contract_account_id.clone())
        //         .call(|adder| adder.inc(increase));
        //     let inc_result = client.call(&ink_e2e::eve(), inc_message, 0, None).await;
        //     assert!(inc_result.is_ok());

        //     // Execute `get` message of `accumulator` contract
        //     let get_result = client
        //         .call_dry_run(&ink_e2e::eve(), &get_message, 0, None)
        //         .await;
        //     assert_eq!(get_result.return_value(), init_value + increase);
        //     Ok(())
        // }
    }
}
