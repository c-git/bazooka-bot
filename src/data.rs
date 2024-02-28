use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::Context;
use shuttle_persist::PersistInstance;
use tracing::error;

#[derive(Debug)]
/// User data, which is stored and accessible in all command invocations
pub struct Data {
    internal: Arc<Mutex<InternalData>>,
    persist: PersistInstance,
}

/// Stores the data used by the application
#[derive(Debug, serde::Serialize, serde::Deserialize, Default)]
struct InternalData {
    message: String,
    count: u16,
}

impl Data {
    const DATA_KEY: &'static str = "internal_data";
    fn internal_data(&self) -> anyhow::Result<MutexGuard<'_, InternalData>> {
        match self.internal.lock() {
            Ok(guard) => Ok(guard),
            Err(e) => anyhow::bail!("failed to lock mutex because '{e}"),
        }
    }
    pub fn set_message(&self, new_message: String) -> anyhow::Result<()> {
        let mut guard = self.internal_data()?;
        guard.message = new_message;
        guard.count += 1;

        // TODO 3: Make save periodic instead of on every change
        self.save(&guard)?;
        Ok(())
    }
    pub fn message(&self) -> anyhow::Result<String> {
        let guard = self.internal_data()?;
        Ok(format!("#{} - {}", guard.count, guard.message))
    }

    pub(crate) fn new(persist: PersistInstance) -> Self {
        let internal = Arc::new(Mutex::new(
            match persist.load::<InternalData>(Self::DATA_KEY) {
                Ok(data) => data,
                Err(e) => {
                    error!("failed to load data: {e}");
                    Default::default()
                }
            },
        ));
        Data { persist, internal }
    }

    fn save(&self, value: &InternalData) -> anyhow::Result<()> {
        self.persist
            .save(Self::DATA_KEY, value)
            .context("failed to save data")?;
        Ok(())
    }
}
