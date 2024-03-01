//! This module stores the business logic of the application.
//! In the poise framework we are only given access to an immutable
//! reference to the data so we need to use a mutex to be able to make
//! changes, and as a result do not get the compiler checks against concurrent
//! access and relying on the mutex can lead to deadlocks. The problem gets worse
//! if we have functions hold the mutex for a long time and possibly call other functions that
//! then call other functions that try to lock the mutex. To prevent this we store
//! functions that need to take the lock on the mutex inside of modules called
//! `protected_ops`` with the rule that they may not call any other functions in the same module

use std::{
    sync::{Arc, Mutex, MutexGuard},
    time::Instant,
};

use anyhow::Context as _;
use shuttle_persist::PersistInstance;
use tracing::{error, info};

use self::unranked::Unranked;

pub(crate) mod unranked;
pub(crate) mod user_serde;

#[derive(Debug)]
/// User data, which is stored and accessible in all command invocations
pub struct Data {
    internal: Arc<Mutex<InternalData>>,
    pub start_instant: Instant,
    persist: PersistInstance,
}

/// Stores the data used by the application
#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct InternalData {
    unranked: Unranked,
}

impl Data {
    const DATA_KEY: &'static str = "internal_data";
    fn internal_data_guard(&self) -> anyhow::Result<MutexGuard<'_, InternalData>> {
        match self.internal.lock() {
            Ok(guard) => Ok(guard),
            Err(e) => anyhow::bail!("failed to lock mutex because '{e}"),
        }
    }

    pub fn new(persist: PersistInstance) -> Self {
        let internal = Arc::new(Mutex::new(
            match persist.load::<InternalData>(Self::DATA_KEY) {
                Ok(data) => {
                    info!("Data Loaded");
                    data
                }
                Err(e) => {
                    error!("failed to load data: {e}");
                    Default::default()
                }
            },
        ));
        Data {
            persist,
            internal,
            start_instant: Instant::now(),
        }
    }

    fn save(&self, value: &InternalData) -> anyhow::Result<()> {
        // TODO 3: Make save periodic instead of on every change
        self.persist
            .save(Self::DATA_KEY, value)
            .context("failed to save data")?;
        info!("Data Saved");
        Ok(())
    }
}
