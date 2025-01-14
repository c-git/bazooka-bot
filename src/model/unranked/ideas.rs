use crate::{
    config::SharedConfig,
    model::{one_based_id::OneBasedId, user_serde::UserIdNumber},
};
use anyhow::{bail, Context as _};
use poise::serenity_prelude::CacheHttp;
use std::fmt::Display;
use tracing::{info, warn};

pub mod protected_ops;

pub type IdeaId = OneBasedId;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
/// If there are any Ideas above a threshold passed then it is guaranteed that the first one returned will also match the output of leading
pub struct Ideas {
    data: Vec<Idea>,

    /// All ideas with this many votes or less will be removed during reset
    pub discard_threshold: usize,
}
#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, PartialEq, Eq)]
pub struct Idea {
    creator: UserIdNumber,
    pub description: String,
    voters: Vec<UserIdNumber>,
}

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
        let votes = match self.voters.len() {
            0 => "No votes".to_string(),
            1 => "1 vote".to_string(),
            x => format!("{x} votes"),
        };
        write!(f, "{} ({votes})", self.description)
    }
}
impl Display for Ideas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Some(leading_index) = self.leading().map(|x| x.0) else {
            // If there is no leading so there is also no data, no action needed
            debug_assert!(self.data.is_empty());
            return Ok(());
        };
        for (i, idea) in self.data.iter().enumerate() {
            if i == leading_index {
                writeln!(f, "**{}. {idea}**", i + 1)?
            } else {
                writeln!(f, "{}. {idea}", i + 1)?
            }
        }
        Ok(())
    }
}

impl Ideas {
    const DATA_KEY: &'static str = "ideas";
    pub const DISPLAY_TITLE: &'static str = "# Unranked Ideas";
    const DEFAULT_DISCARD_THRESHOLD: usize = 2;
    pub fn add(&mut self, user_id_number: UserIdNumber, description: String) {
        let value = Idea::new(user_id_number, description);
        self.data.push(value);
    }

    pub async fn verbose_display(&self, cache_http: impl CacheHttp) -> anyhow::Result<String> {
        use std::fmt::Write as _;
        let mut result = String::new();
        writeln!(
            result,
            "__Discard Threshold: {}__\n",
            self.discard_threshold
        )?;
        let Some(leading_index) = self.leading().map(|x| x.0) else {
            // If there is no leading so there is also no data, no further action needed
            debug_assert!(self.data.is_empty());
            return Ok(result);
        };
        for (i, idea) in self.data.iter().enumerate() {
            writeln!(
                result,
                "{2}{3}{0}. {idea}{3}{2} Suggested by: `{1}`",
                i + 1,
                idea.creator.to_user(&cache_http).await?.name,
                if idea.voters.len() > self.discard_threshold {
                    "__"
                } else {
                    ""
                },
                if leading_index == i { "**" } else { "" }
            )?;
            if let Some(voters) = idea.voters_as_string(&cache_http).await? {
                writeln!(result, "Voters: {voters}")?;
            } else {
                writeln!(result, "[No voters]")?;
            }
            writeln!(result)?; // Add separating line
        }
        writeln!(
            result,
            "_Bold is leading idea and underlined is above threshold_"
        )?;
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
    pub fn remove(
        &mut self,
        id: IdeaId,
        user_id_number: UserIdNumber,
        allow_remove_other: bool,
    ) -> anyhow::Result<Idea> {
        let Some(idea) = self.data.get(id.as_index()) else {
            return Err(self.err_invalid_id(id));
        };

        // Confirm this user created the Idea
        if idea.creator != user_id_number && !allow_remove_other {
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
        // shared_config.persist.data_load_or_default(Self::DATA_KEY)
        // shared_config
        //     .persist
        //     .data_load_or_migration(Self::DATA_KEY, crate::migration::migrate_old_ideas)
        todo!("Load ideas from DB")
    }

    /// If any ideas exist it returns the idea along with its index that has the most votes and appears earliest
    pub fn pop_leading(&mut self) -> Option<Idea> {
        let lead_index = self.leading()?.0;
        let idea_id = IdeaId::from_index(lead_index);
        Some(
            self.remove(idea_id, Default::default(), true)
                .expect("we just checked that this ID exists"),
        )
    }

    /// If any ideas exist it returns the idea along with its index that has the most votes and appears earliest
    pub fn leading(&self) -> Option<(usize, &Idea)> {
        let mut result = (0, self.data.first()?);
        for x in self.data.iter().enumerate().skip(1) {
            if result.1.voters.len() < x.1.voters.len() {
                result = x;
            }
        }
        Some(result)
    }

    /// Discards all ideas at or below the threshold and clears the votes of the remaining ideas
    /// The order of the ideas after reset is guaranteed to be sorted by their previously vote counts
    /// and still in the order they appeared otherwise. The previously leading ideas is guaranteed to
    // be the first one if it existed
    pub fn reset_with_threshold(&mut self) {
        // Sort ideas in required order (see doc string)
        self.data.sort_by_key(|idea| -(idea.voters.len() as i32)); // Rev wouldn't work because we need to keep them in inserted order

        // Remove ideas at or below the line resetting the votes on the rest
        self.data.retain_mut(|idea| {
            if idea.voters.len() > self.discard_threshold {
                idea.voters.clear();
                true
            } else {
                false
            }
        });
    }
}

impl Default for Ideas {
    fn default() -> Self {
        Self {
            data: Default::default(),
            discard_threshold: Self::DEFAULT_DISCARD_THRESHOLD,
        }
    }
}

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

    impl From<(Vec<Idea>, usize)> for Ideas {
        fn from((data, discard_threshold): (Vec<Idea>, usize)) -> Self {
            Self {
                data,
                discard_threshold,
            }
        }
    }

    impl From<(Vec<(&str, Vec<u64>)>, usize)> for Ideas {
        fn from((data, discard_threshold): (Vec<(&str, Vec<u64>)>, usize)) -> Self {
            (
                data.into_iter().map(Into::into).collect::<Vec<Idea>>(),
                discard_threshold,
            )
                .into()
        }
    }

    #[test]
    fn empty_ideas() {
        for i in 0..10 {
            let ideas = Ideas {
                discard_threshold: i,
                ..Default::default()
            };
            assert!(ideas.leading().is_none());
        }
    }

    #[rstest]
    #[case::single_point_over((vec![
            ("only", vec![1,2,3,4])
        ],3).into(),
        Some("only"), vec!["only"])]
    #[case::single_point_under((vec![
            ("only", vec![1,2,3,4])
        ],4).into(),
        Some("only"), vec![])]
    #[case::only_one_over((vec![
            ("first", vec![1,2,3,4]),("second", vec![1,2,3])
        ],3).into(),
        Some("first"), vec!["first"])]
    #[case::pair_equal((vec![
            ("first", vec![1,2,3,4]),
            ("second", vec![1,2,3,4])
        ],3).into(),
        Some("first"), vec!["first", "second"])]
    #[case::multiple_equal_((vec![
            ("1st", vec![1,2,3,4]),
            ("2nd", vec![1,2,3,4]),
            ("3rd", vec![1,2,3,4]),
            ("4th", vec![1,2,3,4,5]),
            ("5th", vec![1,2,3,4,5]),
            ("6th", vec![1,2,3,4,5]),
            ("7th", vec![1,2,3]),
            ("8th", vec![]),
            ("9th", vec![1,2,3,4]),
        ],3).into(),
        Some("4th"), vec!["4th", "5th", "6th","1st","2nd","3rd","9th"])]
    fn test_name(
        #[case] mut ideas: Ideas,
        #[case] expected_leading_desc: Option<&str>,
        #[case] expected_above_ideas_desc: Vec<&str>,
    ) {
        let actual_leading_desc = ideas.leading().map(|idea| idea.1.description.clone());

        ideas.reset_with_threshold(); // Do reset to test remaining descriptions and their order

        let actual_above_ideas_desc: Vec<String> =
            ideas.data.into_iter().map(|x| x.description).collect();

        // Convert to str to make input easier

        let actual_leading_desc = actual_leading_desc.as_ref().map(|x| &x[..]);
        let actual_above_ideas_desc: Vec<&str> =
            actual_above_ideas_desc.iter().map(|x| &x[..]).collect();

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
