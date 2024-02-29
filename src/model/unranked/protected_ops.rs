//! This module exists to make it harder to get deadlocks by grouping functions that MUST NOT call each other.
//! Makes use of the fact that it is a sub-module of data to access the private function from there to implement its functionality

use std::sync::MutexGuard;

use poise::serenity_prelude::User;

use super::{super::Data, IdeaId};
use crate::{model::InternalData, Context};

impl Data {
    /// Serves as the link to the private function that returns the guard
    fn guard(&self) -> anyhow::Result<MutexGuard<InternalData>> {
        self.internal_data_guard()
    }

    pub fn unranked_idea_add(&self, user: &User, description: String) -> anyhow::Result<()> {
        let mut guard = self.guard()?;
        guard.unranked.ideas.add(user, description);
        self.save(&guard)?;
        Ok(())
    }

    pub fn unranked_idea_edit(
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

    pub async fn unranked_ideas_as_string(
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
}
