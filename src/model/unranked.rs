//! Groups the functionality related to unranked business logic

use crate::{
    config::SharedConfig,
    model::unranked::{ideas::Ideas, scores::Scores},
};
use std::sync::{Arc, Mutex};

pub mod ideas;
pub mod scores;

pub struct Unranked {
    ideas: Arc<Mutex<Ideas>>,
    scores: Arc<Mutex<Scores>>,
    shared_config: &'static SharedConfig,
}
impl Unranked {
    pub async fn new(shared_config: &'static SharedConfig) -> Self {
        let ideas = Arc::new(Mutex::new(Ideas::new(shared_config).await));
        let scores = Arc::new(Mutex::new(Scores::new(shared_config).await));
        Self {
            ideas,
            scores,
            shared_config,
        }
    }

    fn save<T: serde::Serialize>(&self, key: &str, value: &T) -> anyhow::Result<()> {
        self.shared_config.save_kv(key, value)
    }
}
