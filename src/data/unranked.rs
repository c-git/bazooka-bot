use std::collections::BTreeMap;

use poise::serenity_prelude::UserId;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Unranked {
    ideas: Ideas,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct Ideas {
    data: BTreeMap<IdeaID, Idea>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Idea {
    creator: UserId,
    description: String,
    voters: Vec<UserId>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdeaID(u32);
