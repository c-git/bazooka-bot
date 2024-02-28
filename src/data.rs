use std::sync::{Arc, Mutex};

#[derive(Debug)]
/// User data, which is stored and accessible in all command invocations
pub struct Data {
    message: Arc<Mutex<String>>,
}

impl Data {
    pub fn set_message(&self, new_message: String) -> anyhow::Result<()> {
        let mut guard = match self.message.lock() {
            Ok(guard) => guard,
            Err(e) => anyhow::bail!("failed to lock mutex because '{e}"),
        };
        *guard = new_message;
        Ok(())
    }
    pub fn message(&self) -> anyhow::Result<String> {
        let guard = match self.message.lock() {
            Ok(guard) => guard,
            Err(e) => anyhow::bail!("failed to lock mutex because '{e}"),
        };
        Ok(guard.clone())
    }

    pub(crate) fn new() -> Self {
        Data {
            message: Arc::new(Mutex::new("Default Message".into())),
        }
    }
}
