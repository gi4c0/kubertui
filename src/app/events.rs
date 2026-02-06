use std::time::Duration;

use anyhow::Context;
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::{self, event::Event as CrosstermEvent};
use tokio::sync::mpsc;

use crate::{
    app::{ActiveWindow, main::logs::PodLogs, notification::Notification},
    error::{AppError, AppResult},
};

const TICK_FPS: f64 = 30.0;

pub enum AppEvent {
    Crossterm(CrosstermEvent),
    Tick,
    Focus(ActiveWindow),
    FocusNext,
    FocusPrev,
    Quit,
    SelectNamespace(String),
    PortForward {
        pod_name: String,
        local_port: u16,
        app_port: u16,
        namespace: String,
    },
    ShowLogs(PodLogs),
    ClosePodsList,
    ShowNotification(Notification),
    HideNotification,
}

pub struct EventHandler {
    sender: EventSender,
    receiver: mpsc::UnboundedReceiver<AppEvent>,
}

#[derive(Clone, Debug)]
pub struct EventSender {
    sender: mpsc::UnboundedSender<AppEvent>,
}

impl EventSender {
    pub fn send(&self, message: AppEvent) {
        let _ = self.sender.send(message);
    }
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        let actor = EventTask::new(sender.clone());
        tokio::spawn(async { actor.run().await });

        Self {
            sender: EventSender { sender },
            receiver,
        }
    }

    pub fn sender(&self) -> EventSender {
        self.sender.clone()
    }

    pub async fn next(&mut self) -> AppResult<AppEvent> {
        self.receiver
            .recv()
            .await
            .context("Failed to get event")
            .map_err(AppError::GeneralError)
    }
}

struct EventTask {
    sender: mpsc::UnboundedSender<AppEvent>,
}

impl EventTask {
    fn new(sender: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self { sender }
    }

    async fn run(self) {
        let mut reader = crossterm::event::EventStream::new();
        let tick_rate = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut tick = tokio::time::interval(tick_rate);

        loop {
            let crossterm_event = reader.next().fuse();
            let tick_delay = tick.tick();

            tokio::select! {
                _ = self.sender.closed() => {
                    break;
                }

                _ = tick_delay => {
                    self.send(AppEvent::Tick);
                }

                Some(Ok(evt)) = crossterm_event => {
                    self.send(AppEvent::Crossterm(evt));
                }
            }
        }
    }

    fn send(&self, event: AppEvent) {
        let _ = self.sender.send(event);
    }
}
