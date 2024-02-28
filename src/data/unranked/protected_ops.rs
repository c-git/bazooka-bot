//! This module exists to make it harder to get deadlocks by groups functions that must not call each other.
//! Makes use of the fact that it is a sub-module of data to access the private function from there to implement its functionality

use std::sync::MutexGuard;

use poise::serenity_prelude::User;

use crate::data::{Data, InternalData};

impl Data {
    /// Servers as the link to the private function that returns the guard
    fn guard(&self) -> anyhow::Result<MutexGuard<InternalData>> {
        self.internal_data_guard()
    }

    pub fn unranked_idea_add(&self, user: &User, description: String) -> anyhow::Result<()> {
        let mut guard = self.guard()?;
        guard.unranked.ideas.add(user, description);
        self.save(&guard)?;
        Ok(())
    }
}
