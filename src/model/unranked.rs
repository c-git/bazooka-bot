//! Groups the functionality related to unranked business logic

use std::fmt::Display;

use anyhow::Context as _;
use poise::serenity_prelude::{CacheHttp, User};

use crate::Context;

use super::UserIdNumber;

pub mod protected_ops;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Unranked {
    ideas: Ideas,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone)]
struct Ideas {
    data: Vec<Idea>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone)]
pub struct Idea {
    creator: UserIdNumber,
    description: String,
    voters: Vec<UserIdNumber>,
}

impl Idea {
    fn new(user: &User, description: String) -> Idea {
        Self {
            creator: user.id.into(),
            description,
            voters: Default::default(),
        }
    }

    async fn voters_as_string(&self, cache_http: impl CacheHttp) -> anyhow::Result<String> {
        let mut users_names = Vec::with_capacity(self.voters.len());
        for id in self.voters.iter() {
            users_names.push(
                id.to_user(&cache_http)
                    .await
                    .context("failed to get user from id")?
                    .name,
            );
        }
        Ok(users_names.join(", "))
    }
}

impl Display for Idea {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - ({})", self.description, self.voters.len())
    }
}
impl Display for Ideas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", Self::DISPLAY_HEADER)?;
        for (i, idea) in self.data.iter().enumerate() {
            writeln!(f, "{}. {idea}", i + 1)?
        }
        Ok(())
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
    const DISPLAY_HEADER: &'static str = "# Unranked Ideas";
    fn add(&mut self, user: &User, description: String) {
        let value = Idea::new(user, description);
        self.data.push(value);
    }

    async fn verbose_display(&self, ctx: &Context<'_>) -> anyhow::Result<String> {
        use std::fmt::Write as _;
        let mut result = String::new();
        writeln!(result, "{}\n", Self::DISPLAY_HEADER)?;
        for (i, idea) in self.data.iter().enumerate() {
            writeln!(
                result,
                "{}. {idea} Suggested by: `{}`",
                i + 1,
                idea.creator.to_user(ctx).await?.name
            )?;
            writeln!(result, "` {} `\n", idea.voters_as_string(ctx).await?)?;
        }
        Ok(result)
    }
}
