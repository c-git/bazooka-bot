//! Groups the functionality related to unranked business logic

use std::{collections::BTreeMap, fmt::Display, num::NonZeroUsize};

use anyhow::{bail, Context as _};
use poise::serenity_prelude::{CacheHttp, User};
use tracing::{error, info, warn};

use crate::Context;

use super::UserIdNumber;

pub mod protected_ops;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Unranked {
    ideas: Ideas,
    scores: Scores,
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

pub type ScoreValue = i8;
type ScoresCache = BTreeMap<ScoreValue, Vec<UserName>>;

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
struct UserName(String);

/// Users scores
///
/// Assumes that each user has at most one record
#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Scores {
    pub message: String,
    records: Vec<ScoreRecord>,
    #[serde(skip)]
    cache: Option<ScoresCache>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ScoreRecord {
    user_id_number: UserIdNumber,
    user_name: UserName,
    score: ScoreValue,
}

impl Scores {
    pub fn set_score(&mut self, user: &User, score: ScoreValue) -> anyhow::Result<()> {
        // Generate cache if it doesn't exist so that the code later can assume it already exists for the current data
        self.cache()?;

        // Check if user already exists in the records and update
        // Assumes that the user exits at most once and this is ensured by this being the only way to add a user
        // If invalid data is loaded that breaks this invariant then the output can be unexpected
        let user_id_number: UserIdNumber = user.id.into();
        for record in self.records.iter_mut() {
            if record.user_id_number == user_id_number {
                // User found. Update score if different and update cache
                if record.score != score {
                    let old_score = record.score;

                    // Update user record
                    record.score = score;

                    let user_name = record.user_name.clone();
                    self.remove_user_from_cache(&old_score, &user_name)?;

                    // Add user to new list in cache
                    self.cache()?.entry(score).or_default().push(user_name);
                }
                return Ok(());
            }
        }

        // New user, not found. Create record and update cache
        let user_name: UserName = user.name.clone().into();
        self.records.push(ScoreRecord {
            user_id_number,
            user_name: user_name.clone(),
            score,
        });
        self.cache()?.entry(score).or_default().push(user_name);
        Ok(())
    }

    /// Removes a user from the cache which means this function depends on the cache existing
    ///
    /// Should be called after the source data is updated in case of errors that busting the cache will lead to the data being updated
    fn remove_user_from_cache(
        &mut self,
        score_in_cache: &ScoreValue,
        user: &UserName,
    ) -> Result<(), anyhow::Error> {
        if self.cache.is_none() {
            error!("Attempt to remove from the cache while it does not exist");
            bail!("Internal Error. Please try again.");
        }
        match self.cache()?.get_mut(score_in_cache) {
            Some(users) => {
                // Remove user from their current location
                users.remove_element(user);
                if users.is_empty() {
                    self.cache()?.remove(score_in_cache);
                }
            }
            None => {
                self.cache = None; // Remove corrupted cache
                error!(
                    "Internal error. Cache seems to be out of sync with the data. Cache busted."
                );
                bail!("Internal error. Please try again.");
            }
        };
        Ok(())
    }

    /// Returns a reference to the cache, filling it if it doesn't exist
    fn cache(&mut self) -> anyhow::Result<&mut ScoresCache> {
        if self.cache.is_none() {
            info!(
                "Scores cache is empty going to fill. {} records found",
                self.records.len()
            );
            let mut map: ScoresCache = BTreeMap::new();
            for record in self.records.iter() {
                map.entry(record.score)
                    .or_default()
                    .push(record.user_name.clone());
            }
            self.cache = Some(map);
        }
        Ok(self
            .cache
            .as_mut()
            .expect("value should have just been set if it didn't exist"))
    }

    /// Removes the score if it exists and returns true iff the score was removed
    pub fn remove_score(&mut self, user: &User) -> anyhow::Result<bool> {
        // Generate cache if it doesn't exist so that the code later can assume it already exists for the current data
        self.cache()?;
        let user_id_number = user.id.into();
        let index = self.records.iter().enumerate().find_map(|(i, x)| {
            if x.user_id_number == user_id_number {
                Some(i)
            } else {
                None
            }
        });

        Ok(if let Some(i) = index {
            let record = self.records.remove(i);
            self.remove_user_from_cache(&record.score, &record.user_name)?;
            true
        } else {
            // User not found
            false
        })
    }

    /// Returns a string representation of the scores
    ///
    /// Wasn't able to use Display trait because we need mutable access
    pub fn display(&mut self) -> anyhow::Result<String> {
        use std::fmt::Write as _;
        let mut result = String::new();
        writeln!(result, "# UNRANKED CHALLENGE")?;
        writeln!(result, "{}\nRankings:", self.message)?;
        for (score, users) in self.cache()?.iter().rev() {
            let user_names: Vec<String> = users.iter().map(|x| format!("`{x}`",)).collect();
            writeln!(result, "{} WINS - {}", score, user_names.join(", "))?;
        }
        Ok(result)
    }
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

trait RemoveElement<T: PartialEq> {
    /// Returns true iff the element was found and removed
    fn remove_element(&mut self, element: &T) -> bool;
}

impl<T: PartialEq> RemoveElement<T> for Vec<T> {
    fn remove_element(&mut self, element: &T) -> bool {
        let index = self
            .iter()
            .enumerate()
            .find_map(|(i, x)| if x == element { Some(i) } else { None });
        if let Some(i) = index {
            self.remove(i);
            true
        } else {
            false
        }
    }
}

impl Display for UserName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<S: Into<String>> From<S> for UserName {
    fn from(value: S) -> Self {
        Self(value.into())
    }
}
