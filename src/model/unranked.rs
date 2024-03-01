//! Groups the functionality related to unranked business logic

pub mod ideas;

pub mod scores;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct Unranked {
    ideas: ideas::Ideas,
    scores: scores::Scores,
}
