# DAO
The governance token gives accounts voting power. Yet, the payout is in the native token. 

## Functionality
- Make a proposal
- Vote on a proposal
- Execute a proposal

## Metrics:
- Voting power: voter's balance

## Ideas:
- Different mechanics for voting power
  - Voter wants to use all its tokens?
- Possibility to fund the DAO
- Being able to be funded with other token
- Fund the dao through erc20 token (= governance_token)
- Creator of DAO can determine whether proposal duration is constant
  * or creator determines the minimum duration for proposal on this dao
  * max. allowed duration?
- ProposalId: Hash
- How do people search for a proposal?
  * Proposal number
  * Hash
- How to make a this DAO contract unique (searchable)?
  * Name
  * Name + token
  * Name + token + blocknumber

## Questions:
- Panic or emitting error?
- Option<T> as return value in query messages.
- Interacting with other contracts in rust test.
