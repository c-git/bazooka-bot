//! This module exists to make it harder to get deadlocks by grouping functions that MUST NOT call each other.
//! Makes use of the fact that it is a sub-module of data to access the private function from there to implement its functionality

use std::sync::MutexGuard;

use poise::serenity_prelude::User;

use super::{super::Data, Idea, IdeaId, ScoreValue};
use crate::{model::InternalData, Context};

impl Data {
    /// Serves as the link to the private function that returns the guard
    fn guard(&self) -> anyhow::Result<MutexGuard<InternalData>> {
        self.internal_data_guard()
    }

    pub(crate) fn unranked_idea_add(&self, user: &User, description: String) -> anyhow::Result<()> {
        let mut guard = self.guard()?;
        guard.unranked.ideas.add(user, description);
        self.save(&guard)?;
        Ok(())
    }

    pub(crate) fn unranked_idea_edit(
        &self,
        id: IdeaId,
        user: &User,
        new_description: String,
    ) -> anyhow::Result<()> {
        let mut guard = self.guard()?;
        guard.unranked.ideas.edit(id, user, new_description)?;
        self.save(&guard)?;
        Ok(())
    }

    /// Attempts to remove and return the Idea
    pub(crate) fn unranked_idea_remove(&self, id: IdeaId, user: &User) -> anyhow::Result<Idea> {
        let mut guard = self.guard()?;
        let result = guard.unranked.ideas.remove(id, user)?;
        self.save(&guard)?;
        Ok(result)
    }

    /// Returns true iff a change was made
    pub(crate) fn unranked_idea_change_vote(
        &self,
        id: IdeaId,
        user: &User,
        is_add_vote: bool,
    ) -> anyhow::Result<bool> {
        let mut guard = self.guard()?;
        let result = guard.unranked.ideas.change_vote(id, user, is_add_vote)?;
        self.save(&guard)?;
        Ok(result)
    }

    /// Returns the number of votes changed
    pub(crate) fn unranked_idea_change_vote_all(
        &self,
        user: &User,
        is_add_vote: bool,
    ) -> anyhow::Result<usize> {
        let mut guard = self.guard()?;
        let result = guard.unranked.ideas.change_vote_all(user, is_add_vote);
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

    pub(crate) fn unranked_score_set(&self, user: &User, score: ScoreValue) -> anyhow::Result<()> {
        let mut guard = self.guard()?;
        guard.unranked.scores.set_score(user, score)?;
        self.save(&guard)?;
        Ok(())
    }

    /// Returns true iff score was removed
    pub(crate) fn unranked_score_remove(&self, user: &User) -> anyhow::Result<bool> {
        let mut guard = self.guard()?;
        let result = guard.unranked.scores.remove_score(user)?;
        self.save(&guard)?;
        Ok(result)
    }

    pub(crate) fn unranked_scores_as_string(&self) -> anyhow::Result<String> {
        let mut guard = self.guard()?;
        let result = guard.unranked.scores.display()?;
        self.save(&guard)?;
        Ok(result)
    }
}
