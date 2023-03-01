#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod token {
    use ink::storage::Mapping;

    /// Defines the storage of your contract.
    /// Add new fields to the below struct in order
    /// to add new static storage fields to your contract.
    #[ink(storage)]
    #[derive(Default)]
    pub struct Token {
        /// Stores a single `bool` value on the storage.
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
        allowances: Mapping<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Token {
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply,
            });
            Self {
                total_supply,
                balances,
                allowances: Default::default(),
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, account: AccountId) -> Balance {
            self.balance_of_impl(&account)
        }

        #[inline]
        pub fn balance_of_impl(&self, account: &AccountId) -> Balance {
            self.balances.get(account).unwrap_or_default()
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_impl(&owner, &spender)
        }

        #[inline]
        pub fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            self.transfer_from_to(&caller, &to, value)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            self.approve_from_to(&caller, &spender, value)
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            owner: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&owner, &caller);
            if value > allowance {
                return Err(Error::InsufficientAllowance);
            }
            self.allowances
                .insert((&owner, &caller), &(allowance - value));
            self.transfer_from_to(&owner, &to, value)
        }

        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
        ) -> Result<()> {
            let from_balance = self.balance_of_impl(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }
            let to_balance = self.balance_of_impl(to);
            self.balances.insert(to, &(to_balance + value));
            self.balances.insert(from, &(from_balance - value));
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });
            Ok(())
        }

        fn approve_from_to(
            &mut self,
            owner: &AccountId,
            spender: &AccountId,
            value: Balance,
        ) -> Result<()> {
            let owner_balance = self.balance_of_impl(owner);
            if owner_balance < value {
                return Err(Error::InsufficientBalance);
            }
            let prev_allowance = self.allowance_impl(owner, spender);
            self.allowances
                .insert((owner, spender), &(prev_allowance + value));
            self.env().emit_event(Approval {
                owner: *owner,
                spender: *spender,
                value,
            });
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        use ink::primitives::{Clear, Hash};
        type Event = <Token as ink::reflect::ContractEventBase>::Type;

        fn assert_transfer_event(
            event: &ink::env::test::EmittedEvent,
            expected_from: Option<AccountId>,
            expected_to: Option<AccountId>,
            expected_value: Balance,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::Transfer(Transfer { from, to, value }) = decoded_event {
                assert_eq!(from, expected_from);
                assert_eq!(to, expected_to);
                assert_eq!(value, expected_value);
            } else {
                panic!("expected Transfer event");
            }
            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue {
                    value: b"Token::Transfer",
                    prefix: b"",
                }),
                encoded_into_hash(&PrefixedValue {
                    value: b"Token::Transfer::from",
                    prefix: &expected_from,
                }),
                encoded_into_hash(&PrefixedValue {
                    value: b"Token::Transfer::to",
                    prefix: &expected_to,
                }),
                encoded_into_hash(&PrefixedValue {
                    value: b"Token::Transfer::value",
                    prefix: &expected_value,
                }),
            ];

            let topics = event.topics.clone();
            for (n, (actual_topic, expected_topic)) in
                topics.iter().zip(expected_topics).enumerate()
            {
                let mut topic_hash = Hash::CLEAR_HASH;
                let len = actual_topic.len();
                topic_hash.as_mut()[0..len].copy_from_slice(&actual_topic[0..len]);

                assert_eq!(topic_hash, expected_topic);
            }
        }

        #[ink::test]
        fn new_works() {
            let token = Token::new(100);
            let emitted_event = ink::env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_event.len());
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut token = Token::new(false);
            assert_eq!(token.get(), false);
            token.flip();
            assert_eq!(token.get(), true);
        }

        /// For calculating the event topic hash.
        struct PrefixedValue<'a, 'b, T> {
            pub prefix: &'a [u8],
            pub value: &'b T,
        }

        impl<X> scale::Encode for PrefixedValue<'_, '_, X>
        where
            X: scale::Encode,
        {
            #[inline]
            fn size_hint(&self) -> usize {
                self.prefix.size_hint() + self.value.size_hint()
            }

            #[inline]
            fn encode_to<T: scale::Output + ?Sized>(&self, dest: &mut T) {
                self.prefix.encode_to(dest);
                self.value.encode_to(dest);
            }
        }

        fn encoded_into_hash<T>(entity: &T) -> Hash
        where
            T: scale::Encode,
        {
            use ink::{
                env::hash::{Blake2x256, CryptoHash, HashOutput},
                primitives::Clear,
            };

            let mut result = Hash::CLEAR_HASH;
            let len_result = result.as_ref().len();
            let encoded = entity.encode();
            let len_encoded = encoded.len();
            if len_encoded <= len_result {
                result.as_mut()[..len_encoded].copy_from_slice(&encoded);
                return result;
            }
            let mut hash_output = <<Blake2x256 as HashOutput>::Type as Default>::default();
            <Blake2x256 as CryptoHash>::hash(&encoded, &mut hash_output);
            let copy_len = core::cmp::min(hash_output.len(), len_result);
            result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
            result
        }
    }

    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = TokenRef::default();

            // When
            let contract_account_id = client
                .instantiate("token", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get =
                build_message::<TokenRef>(contract_account_id.clone()).call(|token| token.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = TokenRef::new(false);
            let contract_account_id = client
                .instantiate("token", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get =
                build_message::<TokenRef>(contract_account_id.clone()).call(|token| token.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip =
                build_message::<TokenRef>(contract_account_id.clone()).call(|token| token.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get =
                build_message::<TokenRef>(contract_account_id.clone()).call(|token| token.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
