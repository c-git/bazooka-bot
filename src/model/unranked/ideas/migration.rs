//! Temporary code used to prevent data loss when data structs change

use shuttle_persist::PersistInstance;
use tracing::{info, instrument, warn};

#[derive(Debug, serde::Deserialize)]
struct Ideas {
    data: Vec<crate::model::unranked::ideas::Idea>,
}

#[instrument(skip(persist))]
pub fn migrate_old_ideas(key: &str, persist: &PersistInstance) -> super::Ideas {
    info!("START");
    let result = match persist.load::<self::Ideas>(key) {
        Ok(old) => {
            info!("Found old version performing update to new version");
            crate::model::unranked::ideas::Ideas {
                data: old.data,
                ..Default::default()
            }
        }
        Err(e) => {
            warn!("Failed to load old version. Going to use default for new. Error was: {e:?}");
            Default::default()
        }
    };
    info!("END");
    result
}
