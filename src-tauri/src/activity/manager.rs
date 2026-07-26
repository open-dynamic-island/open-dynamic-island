use super::reducer::{reduce, ActivityState, IslandAction, IslandEffect};
use island_model::{IslandActivity, IslandDisplayContext, IslandSnapshot};
use std::sync::{Mutex, MutexGuard};
use thiserror::Error;

#[derive(Default)]
pub struct ActivityManager {
    inner: Mutex<ActivityState>,
}

impl ActivityManager {
    pub fn snapshot(&self) -> Result<IslandSnapshot, ActivityError> {
        Ok(self.lock()?.snapshot())
    }

    pub fn dispatch(
        &self,
        action: IslandAction,
    ) -> Result<(IslandSnapshot, Vec<IslandEffect>), ActivityError> {
        let mut state = self.lock()?;
        let effects = reduce(&mut state, action);
        Ok((state.snapshot(), effects))
    }

    pub fn activity(&self, id: &str) -> Result<Option<IslandActivity>, ActivityError> {
        Ok(self
            .lock()?
            .activities
            .get(id)
            .map(|record| record.activity.clone()))
    }

    pub fn set_display_context(
        &self,
        display_context: IslandDisplayContext,
    ) -> Result<(), ActivityError> {
        self.lock()?.display_context = display_context;
        Ok(())
    }

    fn lock(&self) -> Result<MutexGuard<'_, ActivityState>, ActivityError> {
        self.inner
            .lock()
            .map_err(|_| ActivityError::StateUnavailable)
    }
}

#[derive(Debug, Error)]
pub enum ActivityError {
    #[error("activity state is temporarily unavailable")]
    StateUnavailable,
}
