use std::{sync::Arc, time::Duration};

use tokio::{sync::Mutex, task::JoinHandle, time::sleep};

#[derive(Debug, Clone)]
pub struct Spinner {
    current_state: Arc<Mutex<u8>>,
    previous_state: Arc<Mutex<u8>>,
    task: Arc<Option<JoinHandle<()>>>,
}

impl Spinner {
    const VALUES: [&str; 4] = ["|", "/", "-", "\\"];
    const SPIN_TICK_DURATION: Duration = Duration::from_millis(100);

    fn get_next_index(previous_index: u8) -> u8 {
        let last_index = (Self::VALUES.len() - 1) as u8;

        if previous_index < last_index {
            return previous_index + 1;
        }

        0
    }

    pub fn get_spin_state(&self) -> Option<&'static str> {
        if self.task.is_none() {
            return None;
        }

        if let Some(task) = self.task.as_ref()
            && task.is_finished()
        {
            return None;
        }

        match self.current_state.try_lock() {
            Ok(current_state) => Some(Self::VALUES[*current_state as usize]),
            _ => match self.previous_state.try_lock() {
                Ok(previous_state) => Some(Self::VALUES[*previous_state as usize]),
                _ => Some(Self::VALUES[0]),
            },
        }
    }

    pub fn new() -> Self {
        let mut spinner = Self {
            current_state: Arc::new(Mutex::new(0)),
            previous_state: Arc::new(Mutex::new(0)),
            task: Arc::new(None),
        };

        let previous_state = spinner.previous_state.clone();
        let current_state = spinner.current_state.clone();

        spinner.task = Arc::new(Some(tokio::spawn(async move {
            loop {
                sleep(Self::SPIN_TICK_DURATION).await;

                {
                    let mut current_state = current_state.lock().await;
                    *current_state = Self::get_next_index(*current_state);
                }

                {
                    let mut previous_state = previous_state.lock().await;
                    *previous_state = Self::get_next_index(*previous_state);
                }
            }
        })));

        spinner
    }

    pub fn stop(&mut self) {
        if let Some(task) = self.task.as_ref() {
            task.abort();
            self.task = Arc::new(None)
        }
    }
}
