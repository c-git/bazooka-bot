//! This module stores the business logic of the application.
//! In the poise framework we are only given access to an immutable
//! reference to the data so we need to use a mutex to be able to make
//! changes, and as a result do not get the compiler checks against concurrent
//! access and relying on the mutex can lead to deadlocks. The problem gets worse
//! if we have functions hold the mutex for a long time and possibly call other functions that
//! then call other functions that try to lock the mutex. To prevent this we store
//! functions that need to take the lock on the mutex inside of modules called
//! `protected_ops`` with the rule that they may not call any other functions in the same module

use std::sync::{Arc, Mutex};

use anyhow::Context as _;
use shuttle_persist::PersistInstance;
use tracing::{error, info};

use crate::config::SharedConfig;

use self::{schedule::ScheduledTasks, unranked::Unranked};

pub mod schedule;
pub mod unranked;
pub mod user_serde;

/// User data, which is stored and accessible in all command invocations, cheap to clone uses an Arc
#[derive(Clone)]
pub struct Data {
    pub inner: Arc<DataInner>,
}

pub struct DataInner {
    pub unranked: Unranked,
    pub ctx: poise::serenity_prelude::Context,
    pub schedule_tasks: Arc<Mutex<ScheduledTasks>>,
    pub shared_config: &'static SharedConfig,
}

impl Data {
    pub fn new(
        shared_config: &'static SharedConfig,
        ctx: poise::serenity_prelude::Context,
    ) -> Self {
        let result = Data {
            inner: Arc::new(DataInner {
                unranked: Unranked::new(shared_config),
                shared_config,
                schedule_tasks: Arc::new(Mutex::new(ScheduledTasks::new(shared_config))),
                ctx,
            }),
        };
        result.schedule_hydrate();
        result
    }

    fn save<T: serde::Serialize>(&self, key: &str, value: &T) -> anyhow::Result<()> {
        self.inner.shared_config.persist.data_save(key, value)
    }
}

pub(crate) trait PersistData {
    fn data_load_or_default<T: for<'a> serde::Deserialize<'a> + Default>(&self, key: &str) -> T;
    fn data_load_or_migration<T, F>(&self, key: &str, f: F) -> T
    where
        T: for<'a> serde::Deserialize<'a>,
        F: FnOnce(&str, &PersistInstance) -> T;
    fn data_save<T: serde::Serialize>(&self, key: &str, value: &T) -> anyhow::Result<()>;
}

impl PersistData for PersistInstance {
    fn data_save<T: serde::Serialize>(&self, key: &str, value: &T) -> anyhow::Result<()> {
        self.save(key, value)
            .with_context(|| format!("failed to save '{key}' data"))?;
        info!("'{key}' data saved");
        Ok(())
    }

    fn data_load_or_default<T: for<'a> serde::Deserialize<'a> + Default>(&self, key: &str) -> T {
        match self.load::<T>(key) {
            Ok(data) => {
                info!("'{key}' data loaded successfully");
                data
            }
            Err(e) => {
                error!("failed to load '{key}' data. Error was: {e}");
                Default::default()
            }
        }
    }

    fn data_load_or_migration<T, F>(&self, key: &str, f: F) -> T
    where
        T: for<'a> serde::Deserialize<'a>,
        F: FnOnce(&str, &PersistInstance) -> T,
    {
        match self.load::<T>(key) {
            Ok(data) => {
                info!("'{key}' data loaded successfully");
                data
            }
            Err(e) => {
                error!("failed to load '{key}' data. Going fall back to attempting migration. Error was: {e}");
                f(key, self)
            }
        }
    }
}
