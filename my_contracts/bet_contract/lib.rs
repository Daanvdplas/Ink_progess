#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod bet_contract {
    use ink::storage::Mapping;

    #[ink(storage)]
    #[derive(Default)]
    pub struct BetContract {
        owner: AccountId,
        party_a: Hash,
        party_b: Hash,
        bets: Mapping<(AccountId, Hash), Balance>,
        total_bets_party_a: Balance,
        total_bets_party_b: Balance,
    }

    #[ink(event)]
    pub struct BetEvent {
        #[ink(topic)]
        account: AccountId,
        #[ink(topic)]
        party: Hash,
        #[ink(topic)]
        amount: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InvalidTeamHash,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl BetContract {
        /// Create a new bet contract which can be bet on.
        #[ink(constructor)]
        pub fn new(party_a: Hash, party_b: Hash) -> Self {
            let owner = Self::env().caller();
            Self {
                owner,
                party_a,
                party_b,
                bets: Mapping::default(),
                total_bets_party_a: Balance::default(),
                total_bets_party_b: Balance::default(),
            }
        }

        /// Place a bet on a party.
        #[ink(message)]
        pub fn bet(&mut self, party: Hash) {
            let caller = self.env().caller();
            let bet = self.env().transferred_value();
            //assert!(bet);
            let existing_bet = self.bets.get((caller, party)).unwrap_or_default();
            self.bets.insert((caller, party), &(existing_bet + bet));
        }

        /// Get total bet amount on specific party.
        #[ink(message)]
        pub fn total_bet_party(&self, party: Hash) -> Result<Balance> {
            let party_a = self.party_a;
            let party_b = self.party_b;
            match party {
                self.party_a=> Ok(self.total_bets_party_a),
                self.party_b => Ok(self.total_bets_party_b),
                _ => Err(Error::InvalidTeamHash),
            }
        }
    }
}
