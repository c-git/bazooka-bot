use std::collections::BTreeMap;

use anyhow::bail;
use tracing::{error, info};

use crate::{
    config::SharedConfig,
    model::user_serde::{UserIdNumber, UserName, UserRecord},
    RemoveElement as _, Resettable,
};

pub mod protected_ops;

pub type ScoreValue = i8;
type ScoresCache = BTreeMap<ScoreValue, Vec<UserName>>;

/// Users scores
///
/// Assumes that each user has at most one record
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Scores {
    pub message: String,
    records: Vec<ScoreRecord>,
    #[serde(skip)]
    cache: Option<ScoresCache>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct ScoreRecord {
    user: UserRecord,
    score: ScoreValue,
}

impl Scores {
    pub const DISPLAY_TITLE: &'static str = "UNRANKED CHALLENGE";
    const DATA_KEY: &'static str = "scores";
    pub fn set_score(&mut self, user: UserRecord, score: ScoreValue) -> anyhow::Result<()> {
        // Generate cache if it doesn't exist so that the code later can assume it already exists for the current data
        self.cache()?;

        // Check if user already exists in the records and update
        // Assumes that the user exits at most once and this is ensured by this being the only way to add a user
        // If invalid data is loaded that breaks this invariant then the output can be unexpected
        for record in self.records.iter_mut() {
            if record.user.id_number == user.id_number {
                // User found. Update score if different and update cache
                if record.score != score {
                    let old_score = record.score;

                    // Update user record
                    record.score = score;

                    let user_name = record.user.name.clone();
                    self.remove_user_from_cache(&old_score, &user_name)?;

                    // Add user to new list in cache
                    self.cache()?.entry(score).or_default().push(user_name);
                }
                return Ok(());
            }
        }

        // New user, not found. Create record and update cache
        let user_name = user.name.clone();
        self.records.push(ScoreRecord { user, score });
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
                    .push(record.user.name.clone());
            }
            self.cache = Some(map);
        }
        Ok(self
            .cache
            .as_mut()
            .expect("value should have just been set if it didn't exist"))
    }

    /// Removes the score if it exists and returns true iff the score was removed
    pub fn remove_score(&mut self, user: &UserRecord) -> anyhow::Result<bool> {
        // Generate cache if it doesn't exist so that the code later can assume it already exists for the current data
        self.cache()?;
        let index = self.records.iter().enumerate().find_map(|(i, x)| {
            if x.user.id_number == user.id_number {
                Some(i)
            } else {
                None
            }
        });

        Ok(if let Some(i) = index {
            let record = self.records.remove(i);
            self.remove_user_from_cache(&record.score, &record.user.name)?;
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
        writeln!(result, "{}\n\nRankings:", self.message)?;
        for (score, users) in self.cache()?.iter().rev() {
            let user_names: Vec<String> = users.iter().map(|x| format!("{x}",)).collect();
            writeln!(result, "{} WINS - {}", score, user_names.join(", "))?;
        }
        Ok(result)
    }

    pub fn set_message(&mut self, user_id_number: UserIdNumber, msg: String) {
        info!(
            "User# {user_id_number} is replacing scores message from {:?} to {msg:?}",
            self.message
        );
        self.message = msg;
    }

    pub async fn new(shared_config: &SharedConfig) -> Self {
        shared_config.load_or_default_kv(Self::DATA_KEY).await
    }
}

impl Resettable for Scores {}
