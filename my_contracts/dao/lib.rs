#![cfg_attr(not(feature = "std"), no_std)]

mod tests;

#[ink::contract]
mod dao {
    use ink::storage::Mapping;

    type Result<T> = core::result::Result<T, Error>;
    type ProposalId = u64;

    // A proposal that can be made with `fn propose`.
    #[derive(scale::Decode, scale::Encode, Debug, PartialEq, Eq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
    )]
    pub struct Proposal {
        pub to: AccountId,
        pub amount: Balance,
        pub start: Timestamp,
        pub end: Timestamp,
        pub finished: bool,
    }

    // The amount of votes on a given `Proposal`.
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
    )]
    pub struct ProposalVotes {
        total_yes: u64,
        total_no: u64,
    }

    // Type of a vote.
    pub enum VoteType {
        Yes,
        No,
    }

    // Contract storage.
    #[ink(storage)]
    pub struct Dao {
        pub proposals: Mapping<ProposalId, Proposal>,
        pub proposal_votes: Mapping<ProposalId, ProposalVotes>,
        pub votes: Mapping<(ProposalId, AccountId), ()>,
        pub created_proposals: ProposalId,
        pub governance_token: AccountId,
        pub quorum: u8,
    }

    #[ink(event)]
    pub struct NewDao {
        #[ink(topic)]
        pub governance_token: AccountId,
        #[ink(topic)]
        pub quorum: u8,
    }

    #[ink(event)]
    pub struct NewProposal {
        #[ink(topic)]
        pub to: AccountId,
        #[ink(topic)]
        pub amount: Balance,
        #[ink(topic)]
        pub duration: u64,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        // The proposed amount must be higher than 0.
        InsufficientProposalAmount,
        // The duration of the proposal is too short.
        ProposalDurationTooShort,
        // Max amount of proposals created in contract.
        MaxContractProposals,
        // Proposal ID does not exist.
        ProposalNotFound,
    }

    impl Dao {
        // Instantiate a new DAO.
        #[ink(constructor, payable)]
        pub fn new(governance_token: AccountId, quorum: u8) -> Self {
            Self::env().emit_event(NewDao {
                governance_token,
                quorum,
            });
            Dao {
                proposals: Mapping::default(),
                proposal_votes: Mapping::default(),
                votes: Mapping::default(),
                created_proposals: 0,
                governance_token,
                quorum,
            }
        }

        // Propose a new proposal.
        #[ink(message)]
        pub fn propose(&mut self, to: AccountId, amount: Balance, duration: u64) -> Result<()> {
            if amount == 0 {
                return Err(Error::InsufficientProposalAmount);
            }
            if duration == 0 {
                return Err(Error::ProposalDurationTooShort);
            }
            let proposal_id = self.create_proposal_id()?;
            // Sanity while developing
            debug_assert!(self.proposals.get(proposal_id).is_none());
            debug_assert!(self.proposal_votes.get(proposal_id).is_none());

            // Create `Proposal`
            let now = self.env().block_timestamp();
            let proposal = Proposal {
                to,
                amount,
                start: now,
                end: (now + duration),
                finished: false,
            };
            self.proposals.insert(proposal_id, &proposal);
            Self::env().emit_event(NewProposal {
                to,
                amount,
                duration,
            });
            Ok(())
        }

        #[inline]
        fn create_proposal_id(&mut self) -> Result<u64> {
            if self.created_proposals.checked_add(1).is_none() {
                return Err(Error::MaxContractProposals);
            }
            self.created_proposals += 1;
            Ok(self.created_proposals)
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
        pub fn get_proposal(&self, proposal_id: ProposalId) -> Result<Proposal> {
            self.proposals
                .get(proposal_id)
                .ok_or(Error::ProposalNotFound)
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

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;

        // use ink_e2e::build_message;

        // type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        // #[ink_e2e::test]
        // async fn new_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        //     let governance_token: AccountId = [0x08; 32].into();
        //     let quorum = 10;
        //     let dao_constructor = DaoRef::new(governance_token, quorum);
        //     let _dao_id = client
        //         .instantiate("dao", &ink_e2e::alice(), dao_constructor, 100, None)
        //         .await
        //         .expect("dao contract instantiation failed")
        //         .account_id;
        //     Ok(())
        // }

        // #[ink_e2e::test]
        // async fn propose_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
        //     let governance_token: AccountId = [0x08; 32].into();
        //     let quorum = 10;
        //     let dao_constructor = DaoRef::new(governance_token, quorum);
        //     let dao_id = client
        //         .instantiate("dao", &ink_e2e::alice(), dao_constructor, 100, None)
        //         .await
        //         .expect("dao contract instantiation failed")
        //         .account_id;
        //     let propose_message =
        //         ink_e2e::build_message::<DaoRef>(dao_id.clone()).call(|dao| dao.propose(ink_e2e::django(), 10, 10));
        //     let start = client.block_timestamp()
        //     let propose_result = client
        //         .call(&ink_e2e::alice(), propose_message, 0, None)
        //         .await;
        //     assert!(propose_result.is_ok());

        //     let get_proposal_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
        //         .call(|dao| dao.get_proposal(decrease));
        //     let get_proposal_result = client
        //         .call(&ink_e2e::alice(), get_proposal_message, 0, None)
        //         .await;
        //     assert_eq!(get_proposal_result.return_value(), Ok(Proposal {
        //         to: ink_e2e::django(),
        //         amount: 10,

        //     })
        //     Ok(())
        // }
    }
}
