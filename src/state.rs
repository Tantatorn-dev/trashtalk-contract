use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ForumState {
    pub messages: Vec<String>,
    pub count: i32,
    pub owner: Addr,
}

pub const FORUM_STATE: Item<ForumState> = Item::new("forum_state");
