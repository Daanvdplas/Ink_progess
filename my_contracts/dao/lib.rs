#![cfg_attr(not(feature = "std"), no_std)]

mod tests;

#[ink::contract]
mod dao {
    use ink::env::{
        call::{build_call, ExecutionInput, Selector},
        CallFlags, DefaultEnvironment,
    };
    use ink::storage::Mapping;

    type Result<T> = core::result::Result<T, Error>;
    type ProposalId = u64;
    type Votes = u128;

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
        pub executed: bool,
    }

    // The amount of votes on a given `Proposal`.
    #[derive(scale::Decode, scale::Encode, Default, Debug, PartialEq, Eq)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
    )]
    pub struct ProposalVotes {
        pub total_yes: Votes,
        pub total_no: Votes,
    }

    // Type of a vote.
    #[derive(scale::Decode, scale::Encode, Clone, Copy)]
    #[cfg_attr(
        feature = "std",
        derive(ink::storage::traits::StorageLayout, scale_info::TypeInfo)
    )]
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
        pub next_proposal_id: ProposalId,
        pub governance_token: AccountId,
        pub quorum: u8,
    }

    #[ink(event)]
    pub struct DaoCreated {
        #[ink(topic)]
        pub governance_token: AccountId,
        #[ink(topic)]
        pub quorum: u8,
    }

    #[ink(event)]
    pub struct ProposalCreated {
        #[ink(topic)]
        pub proposal_id: ProposalId,
        #[ink(topic)]
        pub to: AccountId,
        #[ink(topic)]
        pub amount: Balance,
        #[ink(topic)]
        pub duration: u64,
    }

    #[ink(event)]
    pub struct Vote {
        #[ink(topic)]
        pub proposal_id: ProposalId,
        #[ink(topic)]
        pub who: AccountId,
        #[ink(topic)]
        pub vote_type: VoteType,
        #[ink(topic)]
        pub vote_amount: Votes,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        // The proposed amount must be higher than 0.
        InsufficientProposalAmount,
        // The duration of the proposal is too short.
        ProposalDurationTooShort,
        // Proposal ID does not exist.
        ProposalNotFound,
        // Proposal has been executed.
        ProposalExecuted,
        // Proposal has expired.
        ProposalExpired,
        // Voter has already voted on this proposal.
        AlreadyVoted,
        // Overflow.
        ArithmeticOverflow,
        // No tokens to vote.
        InsufficientBalance,
    }

    impl Dao {
        // Instantiate a new DAO.
        #[ink(constructor, payable)]
        pub fn new(governance_token: AccountId, quorum: u8) -> Self {
            // Self::env().emit_event(DaoCreated {
            //     governance_token,
            //     quorum,
            // });
            Dao {
                proposals: Mapping::default(),
                proposal_votes: Mapping::default(),
                votes: Mapping::default(),
                next_proposal_id: 0,
                governance_token,
                quorum,
            }
        }

        // Propose a new proposal.
        #[ink(message)]
        pub fn propose(&mut self, to: AccountId, amount: Balance, duration: u64) -> Result<()> {
            if amount == 0 {
                ink::env::debug_println!("amount");
                return Err(Error::InsufficientProposalAmount);
            }
            if duration == 0 {
                ink::env::debug_println!("duration");
                return Err(Error::ProposalDurationTooShort);
            }
            let proposal_id = self.create_proposal_id()?;
            self.next_proposal_id += 1;

            ink::env::debug_println!("yeah");
            // Create `Proposal`
            let now = self.env().block_timestamp();
            self.proposals.insert(
                proposal_id,
                &Proposal {
                    to,
                    amount,
                    start: now,
                    end: (now + duration),
                    executed: false,
                },
            );
            self.proposal_votes.insert(
                proposal_id,
                &ProposalVotes {
                    total_yes: 0,
                    total_no: 0,
                },
            );
            // Self::env().emit_event(ProposalCreated {
            //     proposal_id,
            //     to,
            //     amount,
            //     duration,
            // });
            Ok(())
        }

        #[inline]
        fn create_proposal_id(&mut self) -> Result<u64> {
            self.next_proposal_id
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)
        }

        // Vote on a proposal.
        #[ink(message)]
        pub fn vote(&mut self, proposal_id: ProposalId, vote_type: VoteType) -> Result<()> {
            let proposal = match self.proposals.get(proposal_id) {
                Some(proposal) => proposal,
                _ => return Err(Error::ProposalNotFound),
            };
            self.has_executed(&proposal)?;
            self.has_expired(&proposal)?;
            let caller = self.env().caller();
            self.has_voted(proposal_id, caller)?;
            let vote_amount = self.balance_of(caller);
            if vote_amount == 0 {
                return Err(Error::InsufficientBalance);
            }
            self.add_votes(vote_amount, proposal_id, vote_type)?;
            self.votes.insert((&proposal_id, &caller), &());
            // Self::env().emit_event(Vote {
            //     proposal_id,
            //     who: caller,
            //     vote_type,
            //     vote_amount,
            // });
            Ok(())
        }

        #[inline]
        fn add_votes(
            &mut self,
            vote_amount: Balance,
            proposal_id: ProposalId,
            vote_type: VoteType,
        ) -> Result<()> {
            let proposal_votes = self
                .proposal_votes
                .get(proposal_id)
                .unwrap_or_else(|| panic!("Developer is a dickhead"));
            let proposal_votes = match vote_type {
                VoteType::Yes => ProposalVotes {
                    total_yes: proposal_votes
                        .total_yes
                        .checked_add(vote_amount)
                        .ok_or(Error::ArithmeticOverflow)?,
                    total_no: proposal_votes.total_no,
                },
                VoteType::No => ProposalVotes {
                    total_yes: proposal_votes.total_yes,
                    total_no: proposal_votes
                        .total_no
                        .checked_add(vote_amount)
                        .ok_or(Error::ArithmeticOverflow)?,
                },
            };
            self.proposal_votes.insert(proposal_id, &proposal_votes);
            Ok(())
        }

        #[inline]
        fn balance_of(&self, caller: AccountId) -> Balance {
            build_call::<DefaultEnvironment>()
                .call(self.governance_token)
                .gas_limit(0)
                .transferred_value(0)
                .call_flags(CallFlags::default())
                .exec_input(
                    ExecutionInput::new(Selector::new(ink::selector_bytes!("balance_of")))
                        .push_arg(caller),
                )
                .returns::<Balance>()
                .try_invoke()
                .unwrap_or_else(|env_err| {
                    panic!("cross-contract call to erc20 failed due to {:?}", env_err)
                })
                .unwrap_or_else(|lang_err| {
                    panic!("cross-contract call to erc20 failed due to {:?}", lang_err)
                })
        }

        // #[inline]
        // fn total_supply(&self) -> Balance {
        //     build_call::<DefaultEnvironment>()
        //         .call(self.governance_token)
        //         .gas_limit(0)
        //         .transferred_value(0)
        //         .call_flags(CallFlags::default())
        //         .exec_input(ExecutionInput::new(Selector::new(ink::selector_bytes!(
        //             "total_supply"
        //         ))))
        //         .returns::<Balance>()
        //         .try_invoke()
        //         .unwrap_or_else(|env_err| {
        //             panic!("cross-contract call to erc20 failed due to {:?}", env_err)
        //         })
        //         .unwrap_or_else(|lang_err| {
        //             panic!("cross-contract call to erc20 failed due to {:?}", lang_err)
        //         })
        // }

        #[inline]
        fn has_executed(&self, proposal: &Proposal) -> Result<()> {
            if proposal.executed {
                return Err(Error::ProposalExecuted);
            }
            Ok(())
        }

        #[inline]
        fn has_expired(&self, proposal: &Proposal) -> Result<()> {
            if self.env().block_timestamp() >= proposal.end {
                return Err(Error::ProposalExpired);
            }
            Ok(())
        }

        #[inline]
        fn has_voted(&self, proposal_id: ProposalId, voter: AccountId) -> Result<()> {
            if self.votes.get((proposal_id, voter)).is_some() {
                return Err(Error::AlreadyVoted);
            }
            Ok(())
        }

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
        #[ink(message)]
        pub fn get_votes(&self, proposal_id: ProposalId) -> Result<ProposalVotes> {
            self.proposal_votes
                .get(proposal_id)
                .ok_or(Error::ProposalNotFound)
        }

        // Get the amount of time left to vote on a proposal.
        #[ink(message)]
        pub fn get_proposal_end(&self, proposal_id: ProposalId) -> Result<Timestamp> {
            let proposal = self
                .proposals
                .get(proposal_id)
                .ok_or(Error::ProposalNotFound)?;
            Ok(proposal.end)
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;

        use ink_e2e::build_message;

        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        #[ink_e2e::test]
        async fn gets_work(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            let governance_token: AccountId = [0x08; 32].into();
            let quorum = 10;
            let dao_constructor = DaoRef::new(governance_token, quorum);
            let dao_id = client
                .instantiate("dao", &ink_e2e::ferdie(), dao_constructor, 100, None)
                .await
                .expect("dao contract instantiation failed")
                .account_id;
            // Test get_* without a proposal:
            //
            // Get end of proposal
            let get_end = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_proposal_end(1));
            let get_end_result = client
                .call_dry_run(&ink_e2e::alice(), &get_end, 0, None)
                .await;
            assert_eq!(get_end_result.return_value(), Err(Error::ProposalNotFound));
            // Get proposal
            let get_proposal =
                ink_e2e::build_message::<DaoRef>(dao_id.clone()).call(|dao| dao.get_proposal(1));
            let get_proposal_result = client
                .call_dry_run(&ink_e2e::alice(), &get_proposal, 0, None)
                .await;
            assert_eq!(
                get_proposal_result.return_value(),
                Err(Error::ProposalNotFound)
            );
            // Get total votes on proposal
            let get_votes =
                ink_e2e::build_message::<DaoRef>(dao_id.clone()).call(|dao| dao.get_votes(1));
            let get_votes_result = client
                .call_dry_run(&ink_e2e::alice(), &get_votes, 0, None)
                .await;
            assert_eq!(
                get_votes_result.return_value(),
                Err(Error::ProposalNotFound)
            );

            // Test with a proposal:
            //
            // Propose a proposal
            let ferdie_account = ink_e2e::account_id(ink_e2e::AccountKeyring::Ferdie);
            let propose_message = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.propose(ferdie_account.clone(), 10, 10));
            let propose_result = client
                .call(&ink_e2e::alice(), propose_message, 0, None)
                .await;
            assert!(propose_result.is_ok());
            // Get end of proposal
            let get_end = ink_e2e::build_message::<DaoRef>(dao_id.clone())
                .call(|dao| dao.get_proposal_end(1));
            let get_end_result = client
                .call_dry_run(&ink_e2e::alice(), &get_end, 0, None)
                .await;
            assert!(get_end_result.exec_result.result.is_ok());
            // Hacky way due to no timestamp() in e2e_tests yet
            let start = get_end_result
                .return_value()
                .unwrap_or_else(|_| panic!("shouldn't panic"))
                - 10;
            // Get proposal
            let get_proposal =
                ink_e2e::build_message::<DaoRef>(dao_id.clone()).call(|dao| dao.get_proposal(1));
            let get_proposal_result = client
                .call_dry_run(&ink_e2e::alice(), &get_proposal, 0, None)
                .await;
            assert_eq!(
                get_proposal_result.return_value(),
                Ok(Proposal {
                    to: ferdie_account,
                    amount: 10,
                    start,
                    end: start + 10,
                    executed: false,
                })
            );
            // Get total votes on proposal
            let get_votes =
                ink_e2e::build_message::<DaoRef>(dao_id.clone()).call(|dao| dao.get_votes(1));
            let get_votes_result = client
                .call_dry_run(&ink_e2e::alice(), &get_votes, 0, None)
                .await;
            assert_eq!(
                get_votes_result.return_value(),
                Ok(ProposalVotes {
                    total_yes: 0,
                    total_no: 0,
                })
            );
            Ok(())
        }

        // incorrect_proposal_amount
        // incorrect_proposal_duration
        // already_voted
        // no_proposal
        // proposal_expired
        // proposal_executed
        // insufficient_balance
    }
}
