use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};

pub struct BooleanCondvar {
    value: Mutex<bool>,
    condvar: Condvar,
}

impl BooleanCondvar {
    pub fn new() -> Self {
        BooleanCondvar {
            value: Mutex::new(true),
            condvar: Condvar::new(),
        }
    }

    pub fn wait(&self) -> Result<(), PoisonError<MutexGuard<bool>>> {
        let mut value = self.value.lock()?;
        while *value {
            value = self.condvar.wait(value)?;
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<(), PoisonError<MutexGuard<bool>>> {
        let mut value = self.value.lock()?;
        *value = false;
        self.condvar.notify_all();
        Ok(())
    }
}
