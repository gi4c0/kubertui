use std::{sync::Arc, time::Duration};

use tokio::{sync::Mutex, task::JoinHandle, time::sleep};

#[derive(Default, Debug)]
pub struct Spinner {
    current_state: Arc<Mutex<u8>>,
    previous_state: Arc<Mutex<u8>>,
    task: Option<JoinHandle<()>>,
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

    pub fn new() -> Self {
        let mut spinner = Self {
            current_state: Arc::new(Mutex::new(0)),
            previous_state: Arc::new(Mutex::new(0)),
            task: None,
        };

        spinner.start();

        spinner
    }

    pub fn get_spin_state(&self) -> &'static str {
        match self.current_state.try_lock() {
            Ok(current_state) => Self::VALUES[*current_state as usize],
            _ => match self.previous_state.try_lock() {
                Ok(previous_state) => Self::VALUES[*previous_state as usize],
                _ => Self::VALUES[0],
            },
        }
    }

    fn start(&mut self) {
        if self.task.is_some() {
            return;
        }

        let previous_state = self.previous_state.clone();
        let current_state = self.current_state.clone();

        self.task = Some(tokio::spawn(async move {
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
        }));
    }

    pub fn stop(&self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
