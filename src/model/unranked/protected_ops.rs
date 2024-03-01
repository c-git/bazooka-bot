//! This module exists to make it harder to get deadlocks by grouping functions that MUST NOT call each other.
//! Makes use of the fact that it is a sub-module of data to access the private function from there to implement its functionality

use std::sync::MutexGuard;

use super::{
    super::Data,
    ideas::{Idea, IdeaId},
    scores::ScoreValue,
};
use crate::{
    model::{
        user_serde::{UserIdNumber, UserRecord},
        InternalData,
    },
    Context,
};

impl Data {
    /// Serves as the link to the private function that returns the guard
    fn guard(&self) -> anyhow::Result<MutexGuard<InternalData>> {
        self.internal_data_guard()
    }

    pub(crate) fn unranked_idea_add(
        &self,
        user_id_number: UserIdNumber,
        description: String,
    ) -> anyhow::Result<()> {
        let mut guard = self.guard()?;
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
        let mut guard = self.guard()?;
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
        let mut guard = self.guard()?;
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
        let mut guard = self.guard()?;
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
        let mut guard = self.guard()?;
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
            let ideas = { self.guard()?.unranked.ideas.clone() };
            ideas.verbose_display(ctx).await
        } else {
            Ok(self.guard()?.unranked.ideas.to_string())
        }
    }

    pub(crate) fn unranked_score_set(
        &self,
        user: UserRecord,
        score: ScoreValue,
    ) -> anyhow::Result<()> {
        let mut guard = self.guard()?;
        guard.unranked.scores.set_score(user, score)?;
        self.save(&guard)?;
        Ok(())
    }

    /// Returns true iff score was removed
    pub(crate) fn unranked_score_remove(&self, user: &UserRecord) -> anyhow::Result<bool> {
        let mut guard = self.guard()?;
        let result = guard.unranked.scores.remove_score(user)?;
        self.save(&guard)?;
        Ok(result)
    }

    pub(crate) fn unranked_scores_as_string(&self) -> anyhow::Result<String> {
        let mut guard = self.guard()?;
        let result = guard.unranked.scores.display()?;
        // Note we do not save here because only thing that should change in scores is the cache
        Ok(result)
    }

    pub(crate) fn unranked_score_message(
        &self,
        user_id_number: UserIdNumber,
        msg: String,
    ) -> anyhow::Result<()> {
        let mut guard = self.guard()?;
        guard.unranked.scores.set_message(user_id_number, msg);
        self.save(&guard)?;
        Ok(())
    }
}
