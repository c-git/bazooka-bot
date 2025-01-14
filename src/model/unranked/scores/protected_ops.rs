//! This module exists to make it harder to get deadlocks by grouping functions that MUST NOT call each other.
//! Makes use of the fact that it is a sub-module to access the private function to implement its functionality

use std::sync::MutexGuard;

use crate::{
    model::{
        unranked::Unranked,
        user_serde::{UserIdNumber, UserRecord},
    },
    Resettable as _,
};

use super::{ScoreValue, Scores};

impl Unranked {
    /// Serves as the link to the private function that returns the guard
    fn guard_scores(&self) -> anyhow::Result<MutexGuard<Scores>> {
        match self.scores.lock() {
            Ok(guard) => Ok(guard),
            Err(e) => anyhow::bail!("failed to lock mutex because '{e}"),
        }
    }

    fn save_scores(&self, data: &Scores) -> anyhow::Result<()> {
        self.save(Scores::DATA_KEY, data)
    }

    pub fn score_set(&self, user: UserRecord, score: ScoreValue) -> anyhow::Result<()> {
        let mut guard = self.guard_scores()?;
        guard.set_score(user, score)?;
        self.save_scores(&guard)?;
        Ok(())
    }

    /// Returns true iff score was removed
    pub fn score_remove(&self, user: &UserRecord) -> anyhow::Result<bool> {
        let mut guard = self.guard_scores()?;
        let result = guard.remove_score(user)?;
        self.save_scores(&guard)?;
        Ok(result)
    }

    pub fn scores_as_string(&self) -> anyhow::Result<String> {
        let mut guard = self.guard_scores()?;
        let result = guard.display()?;
        // Note that we do not save here because only thing that should change in scores is the cache which doesn't get saved anyway
        Ok(result)
    }

    pub fn scores_message(&self, user_id_number: UserIdNumber, msg: String) -> anyhow::Result<()> {
        let mut guard = self.guard_scores()?;
        guard.set_message(user_id_number, msg);
        self.save_scores(&guard)?;
        Ok(())
    }

    pub fn scores_reset(&self) -> anyhow::Result<()> {
        let mut guard = self.guard_scores()?;
        guard.reset();
        self.save_scores(&guard)?;
        Ok(())
    }
}
