use std::sync::MutexGuard;

use anyhow::Context;
use tracing::{error, info, instrument};

use crate::Data;

use super::{
    Objective, OutcomeCreateScheduledTask, ScheduledTask, ScheduledTaskId, ScheduledTasks,
    UnixTimestamp,
};

impl Data {
    /// Serves as the link to the private function that returns the guard
    fn guard_schedule(&self) -> anyhow::Result<MutexGuard<ScheduledTasks>> {
        match self.inner.schedule_tasks.lock() {
            Ok(guard) => Ok(guard),
            Err(e) => anyhow::bail!("failed to lock mutex because '{e}"),
        }
    }

    fn save_scheduled_tasks(&self, data: &ScheduledTasks) -> anyhow::Result<()> {
        self.save(ScheduledTasks::DATA_KEY, data)
    }

    #[instrument(skip(self))]
    /// Add a new task to the scheduled tasks and replaces if a task with the same objective exists
    pub fn schedule_create_task(
        &self,
        objective: Objective,
        desired_execution_timestamp: UnixTimestamp,
    ) -> anyhow::Result<OutcomeCreateScheduledTask> {
        let mut guard = self.guard_schedule()?;
        let result = guard.create_task(objective, desired_execution_timestamp, self.clone())?;
        self.save_scheduled_tasks(&guard)?;
        Ok(result)
    }

    #[instrument(skip(self))]
    pub fn schedule_cancel_task_by_id(&self, id: ScheduledTaskId) -> anyhow::Result<ScheduledTask> {
        info!("START");
        let mut guard = self.guard_schedule()?;
        let result = guard.cancel_task_by_id(id)?;
        self.save_scheduled_tasks(&guard)?;
        info!("END");
        Ok(result)
    }
    #[instrument(skip(self))]
    pub fn schedule_cancel_task_by_objective(
        &self,
        objective: Objective,
    ) -> anyhow::Result<ScheduledTask> {
        info!("START");
        let mut guard = self.guard_schedule()?;
        let result = guard.cancel_task_by_objective(objective)?;
        self.save_scheduled_tasks(&guard)?;
        info!("END");
        Ok(result)
    }

    #[instrument(skip(self))]
    /// Creates the tasks from the saved data after restarting the application
    pub fn schedule_hydrate(&self) {
        let mut guard = match self
            .guard_schedule()
            .context("failed to get guard for schedule")
        {
            Ok(guard) => guard,
            Err(e) => {
                error!("unable to hydrate because of error: {e:?}");
                return;
            }
        };
        guard.hydrate(self.clone());
    }

    #[instrument(skip(self))]
    pub fn schedule_as_string(&self) -> anyhow::Result<String> {
        let guard = self.guard_schedule()?;
        Ok(guard.to_string())
    }
}
