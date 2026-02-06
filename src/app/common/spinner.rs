use std::{sync::Arc, time::Duration};

use tokio::{sync::Mutex, task::JoinHandle, time::sleep};

#[derive(Debug, Clone, Default)]
pub struct Spinner {
    state: Arc<Mutex<Option<SpinnerState>>>,
}

#[derive(Debug)]
struct SpinnerState {
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

    pub fn is_loading(&self) -> bool {
        match self.state.try_lock() {
            Ok(state) => state.is_some(),
            _ => false,
        }
    }

    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_spin_state(&self) -> &'static str {
        let state = match self.state.try_lock() {
            Ok(state) => state,
            _ => return Self::VALUES[0],
        };

        match state.as_ref() {
            Some(state) => match state.current_state.try_lock() {
                Ok(current_state) => Self::VALUES[*current_state as usize],
                _ => match state.previous_state.try_lock() {
                    Ok(previous_state) => Self::VALUES[*previous_state as usize],
                    _ => Self::VALUES[0],
                },
            },
            None => " ",
        }
    }

    pub fn start(&mut self) {
        let mut state = SpinnerState {
            current_state: Arc::new(Mutex::new(0)),
            previous_state: Arc::new(Mutex::new(0)),
            task: None,
        };

        let previous_state = state.previous_state.clone();
        let current_state = state.current_state.clone();

        state.task = Some(tokio::spawn(async move {
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

        self.state = Arc::new(Mutex::new(Some(state)));
    }

    pub async fn stop(&mut self) {
        loop {
            let mut maybe_state = self.state.lock().await;

            if let Some(state) = maybe_state.as_mut()
                && let Some(task) = &state.task
            {
                task.abort();
                *maybe_state = None;
                break;
            }
        }
    }
}
