#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod dao {
    use scale::{Decode, Encode};

    type ProposalId = u64;

    struct Proposal {
        to: AccountId,
        amount: Balance,
        start: Timestamp,
        end: Timestamp,
        finished: bool,
    }

    struct ProposalVotes {
        total_yes: u64,
        total_no: u64,
    }

    enum VoteType {
        Yes,
        No,
    }

    #[ink(storage)]
    pub struct Dao {
        proposals: Mapping<ProposalId, Proposal>,
        proposal_votes: Mapping<ProposalId, ProposalVotes>,
        votes: Mapping<(ProposalId, AccountId), ()>,
        next_proposal_id: ProposalId,
        quorum: u8,
    }

    enum Error {
        // The proposed amount must be higher than 0.
        InsufficientProposalAmount,
        // The duration of the proposal is too short.
        ProposalDurationTooShort,
    }

    type Result<T> = core::result::Result<T, Error>;

    impl Dao {
        // Instantiate a new DAO.
        //
        // quorum: minimum number of votes a proposal needs to be considered valid.
        #[ink(constructor, payable)]
        pub fn new(quorum: u8) -> Self {
            Dao {
                treasary: Balance,
                proposals: Mapping::default(),
                proposal_votes: Mapping::default(),
                votes: Mapping::default(),
                next_proposal_id: 0,
                quorum,
            }
        }

        // Propose a new proposal.
        //
        // to: the account to which the `amount` must be send to.
        // amount: the amount of tokens the proposer is asking for.
        // duration: the duration of the proposal.
        #[ink(message)]
        pub fn propose(&mut self, to: AccountId, amount: Balance, duration: u64) -> Result<Self> {
            if amount == 0 {
                return Err(Error::InsufficientProposalAmount);
            }
            if duration == 0 {
                return Err(Error::ProposalDurationTooShort);
            }
        }

        // Vote on a proposal.
        // #[ink(message)]
        // pub fn vote(&mut self) {
        //     self.value = !self.value;
        // }

        // Execute a proposal.
        // #[ink(message)]
        // pub fn execute(&mut self) {
        //     self.value = !self.value;
        // }

        // Get the information regarding a proposal.
        #[ink(message)]
        pub fn get_proposal(&self) -> bool {
            self.value
        }

        // Get the total votes regarding a proposal.
        // #[ink(message)]
        // pub fn get_votes(&self) -> bool {
        //     self.value
        // }

        // Get the amount of time left to vote on a proposal.
        // #[ink(message)]
        // pub fn get_votes(&self) -> bool {
        //     self.value
        // }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let dao = Dao::default();
            assert_eq!(dao.get(), false);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut dao = Dao::new(false);
            assert_eq!(dao.get(), false);
            dao.flip();
            assert_eq!(dao.get(), true);
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
            let constructor = DaoRef::default();

            // When
            let contract_account_id = client
                .instantiate("dao", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<DaoRef>(contract_account_id.clone()).call(|dao| dao.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = DaoRef::new(false);
            let contract_account_id = client
                .instantiate("dao", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<DaoRef>(contract_account_id.clone()).call(|dao| dao.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<DaoRef>(contract_account_id.clone()).call(|dao| dao.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<DaoRef>(contract_account_id.clone()).call(|dao| dao.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
