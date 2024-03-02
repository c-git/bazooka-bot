use std::{fmt::Display, num::NonZeroUsize};

use anyhow::{bail, Context as _};
use poise::serenity_prelude::CacheHttp;
use tracing::{info, warn};

use crate::{
    model::{user_serde::UserIdNumber, PersistData as _, SharedConfig},
    Context, Resettable,
};

pub(crate) mod protected_ops;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone)]
/// If there are any Ideas above a threshold passed then it is guaranteed that the first one returned will also match the output of leading
pub struct Ideas {
    data: Vec<Idea>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, PartialEq, Eq)]
pub struct Idea {
    creator: UserIdNumber,
    description: String,
    voters: Vec<UserIdNumber>,
}

/// Stores an ID of an Idea. These are 1 based because that's
/// how it gets displayed because of markdown
#[derive(
    Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy,
)]
pub struct IdeaId(NonZeroUsize);

impl Idea {
    fn new(creator: UserIdNumber, description: String) -> Idea {
        Self {
            creator,
            description,
            voters: Default::default(),
        }
    }

    async fn voters_as_string(&self, cache_http: impl CacheHttp) -> anyhow::Result<Option<String>> {
        if self.voters.is_empty() {
            return Ok(None);
        }
        let mut users_names = Vec::with_capacity(self.voters.len());
        for id in self.voters.iter() {
            users_names.push(
                id.to_user(&cache_http)
                    .await
                    .context("failed to get user from id")?
                    .name,
            );
        }

        Ok(Some(format!("`{}`", users_names.join(", "))))
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    fn change_vote(&mut self, user_id_number: UserIdNumber, is_add_vote: bool) -> bool {
        let user_number: UserIdNumber = user_id_number;
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
        write!(f, "{} ({} votes)", self.description, self.voters.len())
    }
}
impl Display for Ideas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, idea) in self.data.iter().enumerate() {
            writeln!(f, "{}. {idea}", i + 1)?
        }
        Ok(())
    }
}

impl IdeaId {
    fn as_index(&self) -> usize {
        let x: usize = self.0.into();
        x - 1 // Convert back down to 0 based
    }
}

impl Ideas {
    const DATA_KEY: &'static str = "ideas";
    pub const DISPLAY_TITLE: &'static str = "# Unranked Ideas";
    pub fn add(&mut self, user_id_number: UserIdNumber, description: String) {
        let value = Idea::new(user_id_number, description);
        self.data.push(value);
    }

    pub async fn verbose_display(&self, ctx: &Context<'_>) -> anyhow::Result<String> {
        use std::fmt::Write as _;
        let mut result = String::new();
        for (i, idea) in self.data.iter().enumerate() {
            writeln!(
                result,
                "{}. {idea} Suggested by: `{}`",
                i + 1,
                idea.creator.to_user(ctx).await?.name
            )?;
            if let Some(voters) = idea.voters_as_string(ctx).await? {
                writeln!(result, "{voters}")?;
            }
            writeln!(result)?; // Add separating line
        }
        Ok(result)
    }

    pub fn edit(
        &mut self,
        id: IdeaId,
        user_id_number: UserIdNumber,
        new_description: String,
    ) -> anyhow::Result<()> {
        let Some(idea) = self.data.get_mut(id.as_index()) else {
            return Err(self.err_invalid_id(id));
        };

        // Confirm this user created the Idea
        if idea.creator != user_id_number {
            warn!(
                "Request to edit Idea# {id} by user# {user_id_number} but it was created by {}",
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
    pub fn remove(&mut self, id: IdeaId, user_id_number: UserIdNumber) -> anyhow::Result<Idea> {
        let Some(idea) = self.data.get(id.as_index()) else {
            return Err(self.err_invalid_id(id));
        };

        // Confirm this user created the Idea
        if idea.creator != user_id_number {
            warn!(
                "Request to remove Idea# {id} by user {user_id_number} but it was created by {}",
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
    pub fn change_vote(
        &mut self,
        id: IdeaId,
        user_id_number: UserIdNumber,
        is_add_vote: bool,
    ) -> anyhow::Result<bool> {
        let Some(idea) = self.data.get_mut(id.as_index()) else {
            return Err(self.err_invalid_id(id));
        };

        // Action the vote
        let result = idea.change_vote(user_id_number, is_add_vote);
        info!(
            "{} vote for user# {user_id_number} on Idea# {id} result in {}",
            if is_add_vote { "Add" } else { "Remove" },
            if result { "a change" } else { "no change" }
        );
        Ok(result)
    }

    /// Returns the number of votes changed
    pub fn change_vote_all(&mut self, user_id_number: UserIdNumber, is_add_vote: bool) -> usize {
        let mut result = 0;
        for idea in self.data.iter_mut() {
            if idea.change_vote(user_id_number, is_add_vote) {
                result += 1;
            };
        }
        info!(
            "{} vote for user# {user_id_number} on all ideas result in {result} changes",
            if is_add_vote { "Add" } else { "Remove" },
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

    pub(crate) fn new(shared_config: &SharedConfig) -> Self {
        shared_config.persist.data_load_or_default(Self::DATA_KEY)
    }

    /// Returns the idea if any exist that has the most votes and appears earliest
    pub fn leading(&self) -> Option<&Idea> {
        let mut result = self.data.first()?;
        for idea in self.data.iter().skip(1) {
            if result.voters.len() < idea.voters.len() {
                result = idea;
            }
        }
        Some(result)
    }

    /// Returns the list of ideas above the threshold sorted by number of votes then by insertion order
    pub fn above_threshold(&self, threshold: usize) -> Vec<&Idea> {
        let mut result: Vec<&Idea> = self
            .data
            .iter()
            .filter(|idea| idea.voters.len() > threshold)
            .collect();
        result.sort_by_key(|&idea| -(idea.voters.len() as i32)); // Rev wouldn't work because we need to keep them in inserted order
        debug_assert!(
            (|| {
                if result.is_empty() {
                    return true;
                }
                let leading = self.leading().expect(
                    "if result is not empty then at least one value exists so the should be a leading",
                );
                result.first().unwrap() == &leading
            })(),
            "leading and first returned value should be the same if returned list is not empty"
        );
        result
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

impl Resettable for Ideas {}

#[cfg(test)]
mod tests {
    use poise::serenity_prelude::UserId;
    use rstest::rstest;

    use super::*;

    impl UserIdNumber {
        fn new(value: u64) -> Self {
            UserId::new(value).into()
        }
    }

    impl From<(&str, Vec<u64>)> for Idea {
        fn from(value: (&str, Vec<u64>)) -> Self {
            Self {
                creator: UserIdNumber::new(1),
                description: value.0.to_string(),
                voters: value.1.into_iter().map(|x| UserId::new(x).into()).collect(),
            }
        }
    }

    impl From<Vec<Idea>> for Ideas {
        fn from(data: Vec<Idea>) -> Self {
            Self { data }
        }
    }

    impl From<Vec<(&str, Vec<u64>)>> for Ideas {
        fn from(value: Vec<(&str, Vec<u64>)>) -> Self {
            value
                .into_iter()
                .map(Into::into)
                .collect::<Vec<Idea>>()
                .into()
        }
    }

    #[test]
    fn empty_ideas() {
        let ideas: Ideas = Default::default();
        for i in 0..10 {
            assert!(ideas.leading().is_none());
            assert!(ideas.above_threshold(i).is_empty());
        }
    }

    #[rstest]
    #[case::single_point_over(vec![
            ("only", vec![1,2,3,4])
        ].into(),
        3, Some("only"), vec!["only"])]
    #[case::single_point_under(vec![
            ("only", vec![1,2,3,4])
        ].into(),
        4, Some("only"), vec![])]
    #[case::only_one_over(vec![
            ("first", vec![1,2,3,4]),("second", vec![1,2,3])
        ].into(), 
        3, Some("first"), vec!["first"])]
    #[case::pair_equal(vec![
            ("first", vec![1,2,3,4]),
            ("second", vec![1,2,3,4])
        ].into(), 
        3, Some("first"), vec!["first", "second"])]
    #[case::multiple_equal_(vec![
            ("1st", vec![1,2,3,4]),
            ("2nd", vec![1,2,3,4]),
            ("3rd", vec![1,2,3,4]),
            ("4th", vec![1,2,3,4,5]),
            ("5th", vec![1,2,3,4,5]),
            ("6th", vec![1,2,3,4,5]),
            ("7th", vec![1,2,3]),
            ("8th", vec![]),
            ("9th", vec![1,2,3,4]),
        ].into(), 
        3, Some("4th"), vec!["4th", "5th", "6th","1st","2nd","3rd","9th"])]
    fn test_name(
        #[case] ideas: Ideas,
        #[case] threshold: usize,
        #[case] expected_leading_desc: Option<&str>,
        #[case] expected_above_ideas_desc: Vec<&str>,
    ) {
        let actual_leading_desc = ideas.leading().map(|idea| &idea.description[..]);

        let actual_above_ideas_desc: Vec<&str> = ideas
            .above_threshold(threshold)
            .iter()
            .map(|x| &x.description[..])
            .collect();

        assert_eq!(actual_leading_desc, expected_leading_desc, "check leading");
        assert_eq!(
            actual_above_ideas_desc, expected_above_ideas_desc,
            "check above threshold"
        );
        if !actual_above_ideas_desc.is_empty() {
            assert_eq!(
                actual_above_ideas_desc.first().copied(),
                actual_leading_desc,
                "if there is a first it must match leading"
            );
        }
    }
}
