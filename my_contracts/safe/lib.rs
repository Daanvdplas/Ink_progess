#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod safe {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct Safe {
        deposits: Mapping<AccountId, Balance>,
    }

    #[ink(event)]
    pub struct Deposit {
        #[ink(topic)]
        account: AccountId,
        #[ink(topic)]
        deposit: Balance,
        #[ink(topic)]
        total: Balance,
    }

    #[ink(event)]
    pub struct Withdrawal {
        #[ink(topic)]
        account: AccountId,
        #[ink(topic)]
        withdrawal: Balance,
        #[ink(topic)]
        total: Balance,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        amount: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if this account hasn't deposited to this contract before.
        UnknownAccount,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Safe {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                deposits: Mapping::default(),
            }
        }

        // Caller deposits the transfered amount to the contract
        #[ink(message, payable)]
        pub fn deposit(&mut self) {
            let caller = self.env().caller();
            self.deposit_impl(&caller);
        }

        #[inline]
        pub fn deposit_impl(&mut self, account: &AccountId) {
            let deposit = self.env().transferred_value();
            let deposited = self.deposits.get(account).unwrap_or_default();
            let total = deposited + deposit;
            self.deposits.insert(account, &total);
            self.env().emit_event(Deposit {
                account: *account,
                deposit,
                total,
            });
        }

        // Caller withdraws the amount from the contract if its deposited amount is sufficient.
        #[ink(message)]
        pub fn withdraw(&mut self, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            self.withdraw_impl(&caller, amount)
        }

        #[inline]
        pub fn withdraw_impl(&mut self, account: &AccountId, amount: Balance) -> Result<()> {
            let deposited = self.deposits.get(account).unwrap_or_default();
            if amount > deposited {
                return Err(Error::InsufficientBalance);
            }
            self.env().transfer(*account, amount);
            let total = deposited - amount;
            self.deposits.insert(account, &total);
            self.env().emit_event(Withdrawal {
                account: *account,
                withdrawal: amount,
                total,
            });
            Ok(())
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            self.transfer_impl(&caller, &to, amount)
        }

        #[inline]
        pub fn transfer_impl(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            amount: Balance,
        ) -> Result<()> {
            let from_deposited = self.deposits.get(from).unwrap_or_default();
            if amount > from_deposited {
                return Err(Error::InsufficientBalance);
            }
            let to_deposited = self.deposits.get(to).unwrap_or_default();
            if to_deposited == 0 {
                return Err(Error::UnknownAccount);
            }

            // Execute transfer
            self.deposits.insert(from, &(from_deposited - amount));
            self.deposits.insert(to, &(to_deposited + amount));
            self.env().emit_event(Transfer {
                from: *from,
                to: *to,
                amount,
            });
            Ok(())
        }
        /// Returns the total amount the caller has transfered to the contract.
        #[ink(message)]
        pub fn get(&self) -> Balance {
            let caller = self.env().caller();
            self.deposits.get(caller).unwrap_or_default()
        }

        // Returns the total amount that is transfered to the contract.
        #[ink(message)]
        pub fn total(&self) -> Balance {
            self.env().balance()
        }

        // For showcase purposes of a contract in unit testing having the same
        // AccountId as the contract caller.
        #[ink(message)]
        pub fn get_id(&self) -> AccountId {
            self.env().account_id()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn account_id() {
            // This test is made to showcase the fact that a contract
            // in unit tests gets the same AccountId as the caller.
            let accounts = default_accounts();
            set_sender(accounts.alice);
            assert_eq!(accounts.alice, contract_id());
            set_balance(accounts.alice, 100);
            let safe = Safe::new();
            assert_eq!(safe.total(), 100);
            assert_eq!(accounts.alice, safe.get_id());
        }

        #[ink::test]
        fn deposit() {
            // Create the safe contract
            let mut safe = create_contract();
            assert_eq!(safe.total(), 0);
            assert_eq!(safe.get(), Balance::default());

            // Set the caller to Eve
            let accounts = default_accounts();
            set_sender(accounts.eve);
            set_balance(accounts.eve, 1000);

            // Let Eve make a deposit to the contract
            ink::env::pay_with_call!(safe.deposit(), 100);
            assert_eq!(safe.get(), 100);
            assert_eq!(safe.total(), 100);
            assert_eq!(get_balance(accounts.eve), 900);
        }

        #[ink::test]
        fn multiple_deposits() {
            let mut safe = create_contract();
            assert_eq!(safe.total(), 0);
            assert_eq!(safe.get(), Balance::default());

            let accounts = default_accounts();
            set_sender(accounts.eve);
            set_balance(accounts.eve, 1000);

            ink::env::pay_with_call!(safe.deposit(), 100);
            assert_eq!(safe.get(), 100);
            assert_eq!(safe.total(), 100);
            assert_eq!(get_balance(accounts.eve), 900);

            set_sender(accounts.bob);
            set_balance(accounts.bob, 100);

            ink::env::pay_with_call!(safe.deposit(), 50);
            assert_eq!(safe.get(), 50);
            assert_eq!(safe.total(), 150);
            assert_eq!(get_balance(accounts.bob), 50);
        }

        #[ink::test]
        fn deposit_and_withdraw() {
            let mut safe = create_contract();
            assert_eq!(safe.total(), 0);
            assert_eq!(safe.get(), Balance::default());

            let accounts = default_accounts();
            set_sender(accounts.eve);
            set_balance(accounts.eve, 1000);

            ink::env::pay_with_call!(safe.deposit(), 100);
            assert_eq!(safe.get(), 100);
            assert_eq!(safe.total(), 100);
            assert_eq!(get_balance(accounts.eve), 900);

            safe.withdraw(40);
            assert_eq!(safe.get(), 60);
            assert_eq!(safe.total(), 60);
            assert_eq!(get_balance(accounts.eve), 940);

            safe.withdraw(60);
            assert_eq!(safe.get(), 0);
            assert_eq!(safe.total(), 0);
            assert_eq!(get_balance(accounts.eve), 1000);
        }

        #[ink::test]
        fn multiple_deposits_and_withdrawals() {
            let mut safe = create_contract();
            assert_eq!(safe.total(), 0);
            assert_eq!(safe.get(), Balance::default());

            let accounts = default_accounts();
            set_sender(accounts.eve);
            set_balance(accounts.eve, 1000);

            ink::env::pay_with_call!(safe.deposit(), 100);
            assert_eq!(safe.get(), 100);
            assert_eq!(safe.total(), 100);
            assert_eq!(get_balance(accounts.eve), 900);

            set_sender(accounts.bob);
            set_balance(accounts.bob, 100);

            ink::env::pay_with_call!(safe.deposit(), 50);
            assert_eq!(safe.get(), 50);
            assert_eq!(safe.total(), 150);
            assert_eq!(get_balance(accounts.bob), 50);

            set_sender(accounts.eve);
            safe.withdraw(60);
            assert_eq!(safe.get(), 40);
            assert_eq!(safe.total(), 90);
            assert_eq!(get_balance(accounts.eve), 960);

            set_sender(accounts.bob);
            safe.withdraw(40);
            assert_eq!(safe.get(), 10);
            assert_eq!(safe.total(), 50);
            assert_eq!(get_balance(accounts.bob), 90);
        }

        #[ink::test]
        fn invalid_withdraw() {
            let mut safe = create_contract();
            assert_eq!(safe.total(), 0);
            assert_eq!(safe.get(), Balance::default());

            let accounts = default_accounts();
            set_sender(accounts.eve);
            set_balance(accounts.eve, 1000);

            ink::env::pay_with_call!(safe.deposit(), 100);
            assert_eq!(safe.get(), 100);
            assert_eq!(safe.total(), 100);
            assert_eq!(get_balance(accounts.eve), 900);

            safe.withdraw(200);
            assert_eq!(safe.get(), 100);
            assert_eq!(safe.total(), 100);
            assert_eq!(get_balance(accounts.eve), 900);
        }

        #[ink::test]
        fn transfer() {
            let mut safe = create_contract();
            assert_eq!(safe.total(), 0);
            assert_eq!(safe.get(), Balance::default());

            let accounts = default_accounts();
            set_sender(accounts.eve);
            set_balance(accounts.eve, 1000);

            ink::env::pay_with_call!(safe.deposit(), 100);
            assert_eq!(safe.get(), 100);
            assert_eq!(safe.total(), 100);
            assert_eq!(get_balance(accounts.eve), 900);

            set_sender(accounts.bob);
            set_balance(accounts.bob, 100);

            ink::env::pay_with_call!(safe.deposit(), 50);
            assert_eq!(safe.get(), 50);
            assert_eq!(safe.total(), 150);
            assert_eq!(get_balance(accounts.bob), 50);

            set_sender(accounts.eve);
            if safe.transfer(accounts.bob, 50).is_err() {
                panic!("bad");
            }
            assert_eq!(safe.get(), 50);
            set_sender(accounts.bob);
            assert_eq!(safe.get(), 100);
            assert_eq!(safe.total(), 150);
        }

        fn create_contract() -> Safe {
            let accounts = default_accounts();
            set_sender(accounts.alice);
            set_balance(contract_id(), 0);
            Safe::new()
        }

        fn contract_id() -> AccountId {
            ink::env::test::callee::<ink::env::DefaultEnvironment>()
        }

        fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(sender);
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(account_id, balance)
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
        }
    }
}
