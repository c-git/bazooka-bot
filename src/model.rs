//! This module stores the business logic of the application.
//! In the poise framework we are only given access to an immutable
//! reference to the data so we need to use a mutex to be able to make
//! changes, and as a result do not get the compiler checks against concurrent
//! access and relying on the mutex can lead to deadlocks. The problem gets worse
//! if we have functions hold the mutex for a long time and possibly call other functions that
//! then call other functions that try to lock the mutex. To prevent this we store
//! functions that need to take the lock on the mutex inside of modules called
//! `protected_ops`` with the rule that they may not call any other functions in the same module

use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::Context as _;
use poise::serenity_prelude::{CacheHttp, User, UserId};
use shuttle_persist::PersistInstance;
use tracing::{error, info};

use self::unranked::Unranked;

pub mod unranked;

#[derive(Debug)]
/// User data, which is stored and accessible in all command invocations
pub struct Data {
    internal: Arc<Mutex<InternalData>>,
    persist: PersistInstance,
}

/// Stores the data used by the application
#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct InternalData {
    unranked: Unranked,
}

/// Created to use in place of User or UserId from Framework because they
// are not able to be deserialized from Bincode which shuttle-persist uses
#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, Copy, PartialEq, Eq)]
struct UserIdNumber(u64);

impl UserIdNumber {
    async fn to_user(self, cache_http: impl CacheHttp) -> anyhow::Result<User> {
        Ok(self.to_user_id().to_user(cache_http).await?)
    }

    fn to_user_id(self) -> UserId {
        UserId::from(self.0)
    }
}

impl<T: AsRef<UserId>> From<T> for UserIdNumber {
    fn from(value: T) -> Self {
        Self(value.as_ref().get())
    }
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
        Data { persist, internal }
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
