//! Groups the functionality related to unranked business logic

use std::{fmt::Display, num::NonZeroUsize};

use anyhow::{bail, Context as _};
use poise::serenity_prelude::{CacheHttp, User};
use tracing::{info, warn};

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

    pub fn description(&self) -> &str {
        &self.description
    }

    fn change_vote(&mut self, user: &User, is_add_vote: bool) -> bool {
        let user_number: UserIdNumber = user.id.into();
        let position = self.voters.iter().enumerate().find_map(|(i, voter)| {
            if &user_number == voter {
                Some(i)
            } else {
                None
            }
        });
        match (position, is_add_vote) {
            (None, false) | (Some(_), true) => {
                // Already matches no action needed
                false
            }
            (None, true) => {
                // Not present but should be added
                self.voters.push(user_number);
                true
            }
            (Some(idx), false) => {
                // Exits but should be removed
                self.voters.remove(idx);
                true
            }
        }
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

/// Stores an ID of an Idea. These are 1 based because that's
/// how it gets displayed because of markdown
#[derive(
    Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy,
)]
pub struct IdeaId(NonZeroUsize);
impl IdeaId {
    fn as_index(&self) -> usize {
        let x: usize = self.0.into();
        x - 1 // Convert back down to 0 based
    }
}

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

    fn edit(&mut self, id: IdeaId, user: &User, new_description: String) -> anyhow::Result<()> {
        let Some(idea) = self.data.get_mut(id.as_index()) else {
            return Err(self.err_invalid_id(id));
        };

        // Confirm this user created the Idea
        if idea.creator != user.id.into() {
            warn!(
                "Request to edit Idea# {id} by user {} but it was created by {}",
                user.id,
                idea.creator.to_user_id()
            );
            bail!("Failed to edit Idea# {id} because you didn't create it.")
        }

        info!(
            "Replacing Idea# {id} From {:?} to {:?}",
            idea.description, new_description
        );
        idea.description = new_description;
        Ok(())
    }

    /// Attempts to remove the Idea and return it
    fn remove(&mut self, id: IdeaId, user: &User) -> anyhow::Result<Idea> {
        let Some(idea) = self.data.get(id.as_index()) else {
            return Err(self.err_invalid_id(id));
        };

        // Confirm this user created the Idea
        if idea.creator != user.id.into() {
            warn!(
                "Request to remove Idea# {id} by user {} but it was created by {}",
                user.id,
                idea.creator.to_user_id()
            );
            bail!("Failed to remove Idea# {id} because you didn't create it.")
        }

        // Action the removal
        let result = self.data.remove(id.as_index());
        info!("Removing Idea at ID: {id}. {result:?}");
        Ok(result)
    }

    /// Returns true iff a change was made
    fn change_vote(&mut self, id: IdeaId, user: &User, is_add_vote: bool) -> anyhow::Result<bool> {
        let Some(idea) = self.data.get_mut(id.as_index()) else {
            return Err(self.err_invalid_id(id));
        };

        // Action the vote
        let result = idea.change_vote(user, is_add_vote);
        info!(
            "{} vote for user {:?} on Idea# {id} result in {}",
            if is_add_vote { "Add" } else { "Remove" },
            user.name,
            if result { "a change" } else { "no change" }
        );
        Ok(result)
    }

    /// Returns the number of votes changed
    fn change_vote_all(&mut self, user: &User, is_add_vote: bool) -> usize {
        let mut result = 0;
        for idea in self.data.iter_mut() {
            if idea.change_vote(user, is_add_vote) {
                result += 1;
            };
        }
        info!(
            "{} vote for user {:?} on all ideas result in {result} changes",
            if is_add_vote { "Add" } else { "Remove" },
            user.name,
        );
        result
    }

    fn err_invalid_id(&self, id: IdeaId) -> anyhow::Error {
        anyhow::format_err!(
            "ID: {id} is not a valid ID. {}",
            if self.data.is_empty() {
                "There are NO ideas.".to_string()
            } else {
                format!("Valid IDs are 1..{}", self.data.len())
            }
        )
    }
}

impl From<NonZeroUsize> for IdeaId {
    fn from(value: NonZeroUsize) -> Self {
        Self(value)
    }
}

impl Display for IdeaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
