use std::fmt::Display;

use tokio::task::JoinHandle;
use tracing::{info, instrument};

use crate::Data;

use super::PersistData as _;

pub(crate) mod protected_ops;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, Copy)]
pub struct UnixTimestamp(i32);
impl UnixTimestamp {
    pub(crate) fn new(value: i32) -> Self {
        Self(value)
    }
}

impl Display for UnixTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<t:{0}:F> <t:{0}:R>", self.0)
    }
}

/// Scheduled Tasks
///
/// Stores the info needed to recreate these tasks if application is restarted
#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct ScheduledTasks {
    data: Vec<ScheduledTask>,
}

impl ScheduledTasks {
    pub(crate) fn new(shared_config: &crate::SharedConfig) -> Self {
        shared_config.persist.data_load_or_default(Self::DATA_KEY)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum Objective {
    UnrankedStartEvent,
}

pub enum OutcomeCreateScheduledTask {
    Created,

    /// Includes the previous timestamp
    Replaced(UnixTimestamp),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ScheduledTask {
    desired_execution_timestamp: UnixTimestamp,
    objective: Objective,
    #[serde(skip)]
    task: Option<JoinHandle<()>>,
}

enum OutcomeSpawnTask {
    SucceededReplaced,
    SucceededFromEmpty,
}

impl ScheduledTask {
    /// Returns true iff it was able to successfully spawn the task
    #[instrument(skip(self, data))]
    fn spawn_task(&mut self, data: Data) -> OutcomeSpawnTask {
        info!("START");
        let had_handle = if let Some(old_handle) = self.task.take() {
            info!("Aborting previous handle for {}", self.objective);
            old_handle.abort();
            true
        } else {
            info!("Spawned previous task to abort");
            false
        };
        self.do_spawn(data);
        let result = if had_handle {
            OutcomeSpawnTask::SucceededReplaced
        } else {
            OutcomeSpawnTask::SucceededFromEmpty
        };
        info!("END");
        result
    }

    /// Spawns a new task and saves the join handle
    /// Previous task should already be aborted and cleared
    /// as any currently stored handle will be lost
    fn do_spawn(&mut self, data: Data) {
        let objective = self.objective;
        let desired_execution_timestamp = self.desired_execution_timestamp;
        debug_assert!(
            self.task.is_none(),
            "task should have been aborted already if it existed"
        );
        self.task = Some(tokio::spawn(async move {
            // TODO 1: Schedule proper task
            info!("Started");
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            match objective {
                Objective::UnrankedStartEvent => {
                    info!("running");
                    data.inner
                        .shared_config
                        .channel_unranked
                        .say(
                            data.inner.ctx.clone(),
                            format!("<t:{0}:F> {0}", desired_execution_timestamp.0),
                        )
                        .await
                        .unwrap();
                }
            }
        }));
    }

    fn new(objective: Objective, desired_execution_timestamp: UnixTimestamp) -> Self {
        Self {
            desired_execution_timestamp,
            objective,
            task: None,
        }
    }
}

impl ScheduledTasks {
    const DATA_KEY: &'static str = "scheduled_tasks";

    #[instrument(skip(self, data))]
    pub fn create_task(
        &mut self,
        objective: Objective,
        desired_execution_timestamp: UnixTimestamp,
        data: Data,
    ) -> OutcomeCreateScheduledTask {
        if let Some(existing) = self.find_task(objective) {
            let prev_timestamp = existing.desired_execution_timestamp;
            existing.desired_execution_timestamp = desired_execution_timestamp;
            existing.spawn_task(data);
            OutcomeCreateScheduledTask::Replaced(prev_timestamp)
        } else {
            let mut task = ScheduledTask::new(objective, desired_execution_timestamp);
            task.spawn_task(data);
            self.data.push(task);
            OutcomeCreateScheduledTask::Created
        }
    }

    pub fn find_task(&mut self, objective: Objective) -> Option<&mut ScheduledTask> {
        self.data
            .iter_mut()
            .find(|task| task.objective == objective)
    }

    /// Creates the tasks from the saved data after restarting the application
    #[instrument(skip(self, data))]
    pub fn hydrate(&mut self, data: Data) {
        info!("START");
        for i in (0..self.data.len()).rev() {
            self.data[i].spawn_task(data.clone());
        }
        info!("END");
    }
}

impl Display for Objective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Objective::UnrankedStartEvent => "UnrankedStartEvent",
            }
        )
    }
}
