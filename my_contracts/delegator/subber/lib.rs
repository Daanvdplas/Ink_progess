#![cfg_attr(not(feature = "std"), no_std)]

pub use self::subber::{Subber, SubberRef};

#[ink::contract]
mod subber {
    use accumulator::AccumulatorRef;
    use ink::env::{
        call::{build_call, ExecutionInput},
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
            let _result = build_call::<DefaultEnvironment>()
                .call(self.sub_contract)
                .call_flags(
                    CallFlags::default()
                        .set_tail_call(true)
                        .set_allow_reentry(true),
                )
                .exec_input(ExecutionInput::new(method_selector.into()).push_arg(-by))
                .returns::<()>()
                .try_invoke();
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test(additional_contracts = "../accumulator/Cargo.toml")]
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

        #[ink_e2e::test(additional_contracts = "../accumulator/Cargo.toml")]
        async fn decrease(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
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

            // Instantiate `subber` contract
            let subber_constructor = SubberRef::new(acc_contract_account_id);
            let subber_contract_account_id = client
                .instantiate("subber", &ink_e2e::alice(), subber_constructor, 0, None)
                .await
                .expect("subber contract instantiation failed")
                .account_id;

            // Build `decrease` message of `subber` contract and execute
            let decrease = 10;
            let dec_message =
                ink_e2e::build_message::<SubberRef>(subber_contract_account_id.clone())
                    .call(|subber| subber.dec(decrease));
            let inc_result = client.call(&ink_e2e::eve(), dec_message, 0, None).await;
            assert!(inc_result.is_ok());

            // Execute `get` message of `accumulator` contract
            let get_result = client
                .call_dry_run(&ink_e2e::eve(), &get_message, 0, None)
                .await;
            assert_eq!(get_result.return_value(), init_value - decrease);
            Ok(())
        }
    }
}
