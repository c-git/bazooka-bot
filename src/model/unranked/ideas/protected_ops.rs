//! This module exists to make it harder to get deadlocks by grouping functions that MUST NOT call each other.
//! Makes use of the fact that it is a sub-module to access the private function to implement its functionality

use std::sync::MutexGuard;

use crate::{
    model::{user_serde::UserIdNumber, Data, InternalData},
    Context,
};

use super::{Idea, IdeaId};

impl Data {
    /// Serves as the link to the private function that returns the guard
    fn guard_idea(&self) -> anyhow::Result<MutexGuard<InternalData>> {
        self.internal_data_guard()
    }

    pub(crate) fn unranked_idea_add(
        &self,
        user_id_number: UserIdNumber,
        description: String,
    ) -> anyhow::Result<()> {
        let mut guard = self.guard_idea()?;
        guard.unranked.ideas.add(user_id_number, description);
        self.save(&guard)?;
        Ok(())
    }

    pub(crate) fn unranked_idea_edit(
        &self,
        id: IdeaId,
        user_id_number: UserIdNumber,
        new_description: String,
    ) -> anyhow::Result<()> {
        let mut guard = self.guard_idea()?;
        guard
            .unranked
            .ideas
            .edit(id, user_id_number, new_description)?;
        self.save(&guard)?;
        Ok(())
    }

    /// Attempts to remove and return the Idea
    pub(crate) fn unranked_idea_remove(
        &self,
        id: IdeaId,
        user_id_number: UserIdNumber,
    ) -> anyhow::Result<Idea> {
        let mut guard = self.guard_idea()?;
        let result = guard.unranked.ideas.remove(id, user_id_number)?;
        self.save(&guard)?;
        Ok(result)
    }

    /// Returns true iff a change was made
    pub(crate) fn unranked_idea_change_vote(
        &self,
        id: IdeaId,
        user_id_number: UserIdNumber,
        is_add_vote: bool,
    ) -> anyhow::Result<bool> {
        let mut guard = self.guard_idea()?;
        let result = guard
            .unranked
            .ideas
            .change_vote(id, user_id_number, is_add_vote)?;
        self.save(&guard)?;
        Ok(result)
    }

    /// Returns the number of votes changed
    pub(crate) fn unranked_idea_change_vote_all(
        &self,
        user_id_number: UserIdNumber,
        is_add_vote: bool,
    ) -> anyhow::Result<usize> {
        let mut guard = self.guard_idea()?;
        let result = guard
            .unranked
            .ideas
            .change_vote_all(user_id_number, is_add_vote);
        self.save(&guard)?;
        Ok(result)
    }

    pub(crate) async fn unranked_ideas_as_string(
        &self,
        ctx: &Context<'_>,
        is_verbose: bool,
    ) -> anyhow::Result<String> {
        if is_verbose {
            // Ideas have to be cloned because the guard cannot be held across the await boundary because it is not send
            // Would be a bad idea to hold it anyway because that could lead to a deadlock
            let ideas = { self.guard_idea()?.unranked.ideas.clone() };
            ideas.verbose_display(ctx).await
        } else {
            Ok(self.guard_idea()?.unranked.ideas.to_string())
        }
    }
}
