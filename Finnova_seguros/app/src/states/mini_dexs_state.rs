use sails_rs::prelude::*;

pub struct MiniDexsState {
    pub owner: ActorId,
    pub vft_contract_id: Option<ActorId>,
    pub min_tokens_to_add: u128,
    pub tokens_per_vara: u128
}