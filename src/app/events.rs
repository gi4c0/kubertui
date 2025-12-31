use anyhow::Context;
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::{self, event::Event as CrosstermEvent};
use tokio::sync::mpsc;

use crate::error::{AppError, AppResult};

pub enum AppEvent {
    Crossterm(CrosstermEvent),
    Quit,
    SelectNamespace(String),
    PortForward {
        pod_name: String,
        local_port: u16,
        app_port: u16,
        namespace: String,
    },
    ClosePodsList,
    ShowNotification(Log),
}

pub enum Log {
    Info(String),
    Warning(String),
    Error(String),
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

        loop {
            let crossterm_event = reader.next().fuse();

            tokio::select! {
                _ = self.sender.closed() => {
                    break;
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
