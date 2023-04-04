#[cfg(test)]
mod test {
    use crate::dao::{Dao, Error, NewDao, NewProposal, Proposal};
    use ink::primitives::AccountId;
    type Event = <Dao as ::ink::reflect::ContractEventBase>::Type;
    type Balance = u128;
    type Timestamp = u64;

    #[ink::test]
    fn create_dao() {
        let governance_token: AccountId = [0x08; 32].into();
        let quorum = 10;
        let dao = create_contract(governance_token, quorum, 1000);
        let emitted_events = get_events();
        assert_eq!(emitted_events.len(), 1);
        assert_new_event(&emitted_events[0], governance_token, quorum);
    }

    #[ink::test]
    fn correct_proposal() {
        let governance_token: AccountId = [0x08; 32].into();
        let quorum = 10;
        let mut dao = create_contract(governance_token, quorum, 1000);
        let accounts = default_accounts();
        // proposal
        set_block_timestamp(50);
        let propose_result = dao.propose(accounts.django, 10, 10);
        assert!(propose_result.is_ok());
        // verify with `get_proposal`
        if let Ok(proposal) = dao.get_proposal(1) {
            assert_proposal(proposal, 50, accounts.django, 10, 10);
        } else {
            panic!("Proposal did not exist while it should");
        }
        // verify with emitted events
        let emitted_events = get_events();
        assert_eq!(emitted_events.len(), 2);
        assert_propose_event(&emitted_events[1], accounts.django, 10, 10);
    }

    #[ink::test]
    fn incorrect_proposal_amount() {
        let governance_token: AccountId = [0x08; 32].into();
        let quorum = 10;
        let mut dao = create_contract(governance_token, quorum, 1000);
        let accounts = default_accounts();
        // proposal
        let propose_result = dao.propose(accounts.django, 0, 10);
        assert!(propose_result.is_err());
        // verify with `get_proposal`
        if let Err(error) = dao.get_proposal(1) {
            assert_eq!(error, Error::ProposalNotFound)
        } else {
            panic!("Proposal should not have exitsted");
        }
        // verify with emitted events
        let emitted_events = get_events();
        assert_eq!(emitted_events.len(), 1);
    }

    #[ink::test]
    fn incorrect_proposal_duration() {
        let governance_token: AccountId = [0x08; 32].into();
        let quorum = 10;
        let mut dao = create_contract(governance_token, quorum, 1000);
        let accounts = default_accounts();
        // proposal
        let propose_result = dao.propose(accounts.django, 10, 0);
        assert!(propose_result.is_err());
        // verify with `get_proposal`
        if let Err(error) = dao.get_proposal(1) {
            assert_eq!(error, Error::ProposalNotFound)
        } else {
            panic!("Proposal should not have exitsted");
        }
        // verify with emitted events
        let emitted_events = get_events();
        assert_eq!(emitted_events.len(), 1);
    }

    // Helper functions for tests
    fn create_contract(governance_token: AccountId, quorum: u8, init_balance: Balance) -> Dao {
        let accounts = default_accounts();
        set_sender(accounts.alice);
        set_balance(contract_id(), init_balance);
        debug_assert_eq!(get_balance(contract_id()), init_balance);
        Dao::new(governance_token, quorum)
    }

    fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
        ink::env::test::default_accounts::<ink::env::DefaultEnvironment>()
    }

    fn set_sender(account: AccountId) {
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(account)
    }

    fn set_balance(account: AccountId, amount: Balance) {
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(account, amount);
    }

    fn contract_id() -> AccountId {
        ink::env::test::callee::<ink::env::DefaultEnvironment>()
    }

    fn get_balance(account: AccountId) -> Balance {
        ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(account)
            .unwrap_or_default()
    }

    fn get_events() -> Vec<ink::env::test::EmittedEvent> {
        ink::env::test::recorded_events().collect::<Vec<_>>()
    }

    fn set_block_timestamp(timestamp: Timestamp) {
        ink::env::test::set_block_timestamp::<ink::env::DefaultEnvironment>(timestamp);
    }

    fn assert_new_event(
        event: &ink::env::test::EmittedEvent,
        desired_gov_token: AccountId,
        desired_quorum: u8,
    ) {
        let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..]);
        if let Ok(Event::NewDao(NewDao {
            governance_token,
            quorum,
        })) = decoded_event
        {
            assert_eq!(governance_token, desired_gov_token);
            assert_eq!(quorum, desired_quorum);
        } else {
            panic!("Decoding of NewProposal event failed")
        }
    }

    fn assert_proposal(
        proposal: Proposal,
        start: Timestamp,
        to: AccountId,
        amount: Balance,
        duration: Timestamp,
    ) {
        assert_eq!(
            proposal,
            Proposal {
                to,
                amount,
                start,
                end: start + duration,
                finished: false,
            }
        )
    }

    fn assert_propose_event(
        event: &ink::env::test::EmittedEvent,
        proposed_to: AccountId,
        proposed_amount: Balance,
        proposed_duration: u64,
    ) {
        let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..]);
        if let Ok(Event::NewProposal(NewProposal {
            to,
            amount,
            duration,
        })) = decoded_event
        {
            assert_eq!(to, proposed_to);
            assert_eq!(amount, proposed_amount);
            assert_eq!(duration, proposed_duration);
        } else {
            panic!("Decoding of NewProposal event failed")
        }
    }
}
