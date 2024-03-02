//! This module exists to make it harder to get deadlocks by grouping functions that MUST NOT call each other.
//! Makes use of the fact that it is a sub-module to access the private function to implement its functionality

use std::sync::MutexGuard;

use crate::{
    model::{unranked::Unranked, user_serde::UserIdNumber},
    Context, Resettable as _,
};

use super::{Idea, IdeaId, Ideas};

impl Unranked {
    fn guard_idea(&self) -> anyhow::Result<MutexGuard<Ideas>> {
        match self.ideas.lock() {
            Ok(guard) => Ok(guard),
            Err(e) => anyhow::bail!("failed to lock mutex because '{e}"),
        }
    }
    fn save_idea(&self, data: &Ideas) -> anyhow::Result<()> {
        self.save(Ideas::DATA_KEY, data)
    }

    pub(crate) fn idea_add(
        &self,
        user_id_number: UserIdNumber,
        description: String,
    ) -> anyhow::Result<()> {
        let mut guard = self.guard_idea()?;
        guard.add(user_id_number, description);
        self.save_idea(&guard)?;
        Ok(())
    }

    pub(crate) fn idea_edit(
        &self,
        id: IdeaId,
        user_id_number: UserIdNumber,
        new_description: String,
    ) -> anyhow::Result<()> {
        let mut guard = self.guard_idea()?;
        guard.edit(id, user_id_number, new_description)?;
        self.save_idea(&guard)?;
        Ok(())
    }

    /// Attempts to remove and return the Idea
    pub(crate) fn idea_remove(
        &self,
        id: IdeaId,
        user_id_number: UserIdNumber,
    ) -> anyhow::Result<Idea> {
        let mut guard = self.guard_idea()?;
        let result = guard.remove(id, user_id_number)?;
        self.save_idea(&guard)?;
        Ok(result)
    }

    /// Returns true iff a change was made
    pub(crate) fn idea_change_vote(
        &self,
        id: IdeaId,
        user_id_number: UserIdNumber,
        is_add_vote: bool,
    ) -> anyhow::Result<bool> {
        let mut guard = self.guard_idea()?;
        let result = guard.change_vote(id, user_id_number, is_add_vote)?;
        self.save_idea(&guard)?;
        Ok(result)
    }

    /// Returns the number of votes changed
    pub(crate) fn idea_change_vote_all(
        &self,
        user_id_number: UserIdNumber,
        is_add_vote: bool,
    ) -> anyhow::Result<usize> {
        let mut guard = self.guard_idea()?;
        let result = guard.change_vote_all(user_id_number, is_add_vote);
        self.save_idea(&guard)?;
        Ok(result)
    }

    pub(crate) async fn ideas_as_string(
        &self,
        ctx: &Context<'_>,
        is_verbose: bool,
    ) -> anyhow::Result<String> {
        if is_verbose {
            // Ideas have to be cloned because the guard cannot be held across the await boundary because it is not send
            // Would be a bad idea to hold it anyway because that could lead to a deadlock
            let ideas = { self.guard_idea()?.clone() };
            ideas.verbose_display(ctx).await
        } else {
            Ok(self.guard_idea()?.to_string())
        }
    }

    pub(crate) fn ideas_reset(&self) -> anyhow::Result<()> {
        let mut guard = self.guard_idea()?;
        guard.reset();
        self.save_idea(&guard)?;
        Ok(())
    }
}
