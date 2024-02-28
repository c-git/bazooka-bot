//! Groups the functionality related to unranked business logic

use std::collections::BTreeMap;

use poise::serenity_prelude::{User, UserId};

pub mod protected_ops;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Unranked {
    ideas: Ideas,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct Ideas {
    data: BTreeMap<IdeaID, Idea>,
    next_id: IdeaID,
}
impl Ideas {
    fn add(&mut self, user: &User, description: String) {
        let next_id = self.next_id;
        self.next_id.increment();
        let value = Idea::new(user, description);
        self.data.insert(next_id, value);
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Idea {
    creator: UserId,
    description: String,
    voters: Vec<UserId>,
}
impl Idea {
    fn new(user: &User, description: String) -> Idea {
        Self {
            creator: user.id,
            description,
            voters: Default::default(),
        }
    }
}

#[derive(
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Clone,
    Copy,
)]
pub struct IdeaID(u32);
impl IdeaID {
    /// Returns the current value and increments in preparation for next time
    fn increment(&mut self) -> Self {
        let result = *self;
        self.0 += 1;
        result
    }
}
