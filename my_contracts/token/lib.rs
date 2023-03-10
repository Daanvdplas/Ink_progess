#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod token {
    use ink::storage::Mapping;

    #[ink::trait_definition]
    pub trait Erc20 {
        #[ink(message)]
        fn total_supply(&self) -> Balance;

        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()>;

        #[ink(message)]
        fn balance_of(&self, account: AccountId) -> Balance;

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()>;

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

        #[ink(message)]
        fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()>;
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Token {
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
    }

    impl Erc20 for Token {
        #[ink(message)]
        fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        fn balance_of(&self, account: AccountId) -> Balance {
            self.balance_of_impl(&account)
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_impl(&owner, &spender)
        }

        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            self.transfer_from_to(&caller, &to, value)
        }

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            self.approve_from_to(&caller, &spender, value)
        }

        #[ink(message)]
        fn transfer_from(&mut self, owner: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&owner, &caller);
            if value > allowance {
                return Err(Error::InsufficientAllowance);
            }
            self.allowances
                .insert((&owner, &caller), &(allowance - value));
            self.transfer_from_to(&owner, &to, value)
        }
    }

    #[ink(impl)]
    impl Token {
        #[inline]
        fn balance_of_impl(&self, account: &AccountId) -> Balance {
            self.balances.get(account).unwrap_or_default()
        }

        #[inline]
        fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
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
}
