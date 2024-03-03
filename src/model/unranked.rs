//! Groups the functionality related to unranked business logic

use std::sync::{Arc, Mutex};

use crate::{
    config::SharedConfig,
    model::{
        unranked::{ideas::Ideas, scores::Scores},
        PersistData as _,
    },
};

pub mod ideas;
pub mod scores;

#[derive(Debug)]
pub struct Unranked {
    ideas: Arc<Mutex<Ideas>>,
    scores: Arc<Mutex<Scores>>,
    shared_config: &'static SharedConfig,
}
impl Unranked {
    pub(crate) fn new(shared_config: &'static SharedConfig) -> Self {
        let ideas = Arc::new(Mutex::new(Ideas::new(shared_config)));
        let scores = Arc::new(Mutex::new(Scores::new(shared_config)));
        Self {
            ideas,
            scores,
            shared_config,
        }
    }

    fn save<T: serde::Serialize>(&self, key: &str, value: &T) -> anyhow::Result<()> {
        self.shared_config.persist.data_save(key, value)
    }
}
