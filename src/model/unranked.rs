//! Groups the functionality related to unranked business logic

use poise::serenity_prelude::{User, UserId};

pub mod protected_ops;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Unranked {
    ideas: Ideas,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct Ideas {
    data: Vec<Idea>,
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

impl Ideas {
    fn add(&mut self, user: &User, description: String) {
        let value = Idea::new(user, description);
        self.data.push(value);
    }
}
