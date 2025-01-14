use super::one_based_id::OneBasedId;
use crate::{commands::do_start_event, Data};
use anyhow::{bail, Context};
use human_time::ToHumanTimeString;
use shuttle_runtime::tokio::{self, task::JoinHandle};
use std::{
    fmt::Display,
    time::{Duration, UNIX_EPOCH},
};
use tracing::{error, info, instrument, warn};

pub mod protected_ops;
pub type ScheduledTaskId = OneBasedId;

#[derive(Debug, serde::Serialize, serde::Deserialize, Default, Clone, Copy)]
pub struct UnixTimestamp(i32);
impl UnixTimestamp {
    pub fn new(value: i32) -> Self {
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
    pub const DISPLAY_TITLE: &'static str = "Scheduled Tasks";
    pub fn new(shared_config: &crate::SharedConfig) -> Self {
        shared_config.load_or_default_kv(Self::DATA_KEY)
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
    pub desired_execution_timestamp: UnixTimestamp,
    pub objective: Objective,
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
    fn spawn_task(&mut self, data: Data) -> anyhow::Result<OutcomeSpawnTask> {
        info!("START");
        let had_handle = if let Some(old_handle) = self.task.take() {
            info!("Aborting previous handle for {}", self.objective);
            old_handle.abort();
            true
        } else {
            info!("No previously spawned task to abort");
            false
        };
        self.do_spawn(data)?;
        let result = if had_handle {
            OutcomeSpawnTask::SucceededReplaced
        } else {
            OutcomeSpawnTask::SucceededFromEmpty
        };
        info!("END");
        Ok(result)
    }

    /// Spawns a new task and saves the join handle
    /// Previous task should already be aborted and cleared
    /// as any currently stored handle will be lost
    #[instrument(skip(self, data) fields(self.objective = %self.objective, self.desired_execution_timestamp = ?self.desired_execution_timestamp))]
    fn do_spawn(&mut self, data: Data) -> anyhow::Result<()> {
        let objective = self.objective;
        debug_assert!(
            self.task.is_none(),
            "task should have been aborted already if it existed"
        );
        let seconds_since_epoch = UNIX_EPOCH
            .elapsed()
            .context("failed to get timestamp. System date before Unix Epoch?")?
            .as_secs();
        let seconds_since_epoch: i32 = seconds_since_epoch
            .try_into()
            .context("failed to convert system time as seconds since epoch into i32")?;
        info!(seconds_since_epoch);
        let timestamp_now = UnixTimestamp::new(seconds_since_epoch);
        info!("timestamp_now={timestamp_now:?}");
        let seconds_to_desired = self.desired_execution_timestamp.0 - timestamp_now.0;
        info!(seconds_to_desired);
        if seconds_to_desired <= 0 {
            let duration_in_past = Duration::from_secs(seconds_to_desired.unsigned_abs() as _);
            let err_msg = format!(
                "unable to schedule task because duration is {} in the past",
                duration_in_past.to_human_time_string()
            );
            error!(err_msg);
            bail!(err_msg);
        }
        let sleep_duration = Duration::from_secs(
            seconds_to_desired
                .try_into()
                .context("failed to convert seconds to sleep into a duration")?,
        );

        self.task = Some(tokio::spawn(async move {
            // Sleep until it's time to work
            info!(
                "spawned event started, going to sleep for {}",
                sleep_duration.to_human_time_string()
            );
            tokio::time::sleep(sleep_duration).await;
            info!("sleeping task has woken up with objective: {objective}");

            // Do the objective
            let cmd_result = match objective {
                Objective::UnrankedStartEvent => {
                    do_start_event(
                        data.inner.ctx.clone(),
                        data.inner.shared_config.channel_unranked,
                        &data,
                    )
                    .await
                }
            };

            // Check result of objective
            match cmd_result {
                Ok(()) => info!("objective accomplished"),
                Err(e) => error!("failed to accomplish objective with error: {e:?}"),
            }

            // Remove task from list (We can only do this as we are running from a different task as the mutex is locked rn and we would create a deadlock if this were on the same execution path)
            if let Err(e) = data.schedule_cancel_task_by_objective(objective) {
                error!("failed to remove the task from with error: {e:?}");
            }
        }));
        Ok(())
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
    ) -> anyhow::Result<OutcomeCreateScheduledTask> {
        if let Some(existing) = self.find_task(objective) {
            let prev_timestamp = existing.desired_execution_timestamp;
            existing.desired_execution_timestamp = desired_execution_timestamp;
            existing.spawn_task(data)?;
            Ok(OutcomeCreateScheduledTask::Replaced(prev_timestamp))
        } else {
            let mut task = ScheduledTask::new(objective, desired_execution_timestamp);
            task.spawn_task(data)?;
            self.data.push(task);
            Ok(OutcomeCreateScheduledTask::Created)
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
            match self.data[i].spawn_task(data.clone()) {
                Ok(_) => (),
                Err(e) => {
                    error!(
                            "Removing task with objective {} because failed to hydrate with error: {e:?}",
                            self.data[i].objective
                        );
                    self.data.remove(i);
                }
            };
        }
        info!("END");
    }

    #[instrument(skip(self))]
    pub fn cancel_task_by_id(&mut self, id: ScheduledTaskId) -> anyhow::Result<ScheduledTask> {
        info!("START");
        let index = id.as_index();
        if index < self.data.len() {
            info!("ENDING with removal");
            Ok(self.data.remove(index))
        } else {
            warn!("ENDING with out of bounds");
            bail!("Invalid ID received for cancel of {id}");
        }
    }

    #[instrument(skip(self))]
    pub fn cancel_task_by_objective(
        &mut self,
        objective: Objective,
    ) -> anyhow::Result<ScheduledTask> {
        info!("START");
        let index = self.data.iter().enumerate().find_map(|(i, task)| {
            if task.objective == objective {
                Some(i)
            } else {
                None
            }
        });
        if let Some(index) = index {
            info!("ENDING with removal");
            Ok(self.data.remove(index))
        } else {
            warn!("ENDING with objective not found");
            bail!("Unable to find any scheduled task with objective: {objective}");
        }
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

impl Display for ScheduledTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Objective: {}, Scheduled for {}",
            self.objective, self.desired_execution_timestamp
        )
    }
}

impl Display for ScheduledTasks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, task) in self.data.iter().enumerate() {
            writeln!(f, "{}. {}", i + 1, task)?;
        }
        Ok(())
    }
}
