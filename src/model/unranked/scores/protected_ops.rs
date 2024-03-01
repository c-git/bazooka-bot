//! This module exists to make it harder to get deadlocks by grouping functions that MUST NOT call each other.
//! Makes use of the fact that it is a sub-module to access the private function to implement its functionality

use std::sync::MutexGuard;

use crate::model::{
    user_serde::{UserIdNumber, UserRecord},
    Data, InternalData,
};

use super::ScoreValue;

impl Data {
    /// Serves as the link to the private function that returns the guard
    fn guard_scores(&self) -> anyhow::Result<MutexGuard<InternalData>> {
        self.internal_data_guard()
    }

    pub(crate) fn unranked_score_set(
        &self,
        user: UserRecord,
        score: ScoreValue,
    ) -> anyhow::Result<()> {
        let mut guard = self.guard_scores()?;
        guard.unranked.scores.set_score(user, score)?;
        self.save(&guard)?;
        Ok(())
    }

    /// Returns true iff score was removed
    pub(crate) fn unranked_score_remove(&self, user: &UserRecord) -> anyhow::Result<bool> {
        let mut guard = self.guard_scores()?;
        let result = guard.unranked.scores.remove_score(user)?;
        self.save(&guard)?;
        Ok(result)
    }

    pub(crate) fn unranked_scores_as_string(&self) -> anyhow::Result<String> {
        let mut guard = self.guard_scores()?;
        let result = guard.unranked.scores.display()?;
        // Note we do not save here because only thing that should change in scores is the cache
        Ok(result)
    }

    pub(crate) fn unranked_score_message(
        &self,
        user_id_number: UserIdNumber,
        msg: String,
    ) -> anyhow::Result<()> {
        let mut guard = self.guard_scores()?;
        guard.unranked.scores.set_message(user_id_number, msg);
        self.save(&guard)?;
        Ok(())
    }
}
