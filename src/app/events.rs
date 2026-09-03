use std::time::Duration;

use ::crossterm::event::EventStream;
use anyhow::Context;
use futures::{FutureExt, StreamExt};
use ratatui::crossterm::event::Event as CrosstermEvent;
use tokio::sync::mpsc;

use crate::{
    app::{main_window::explorer::ExplorerKind, modal::Modal, notification::Notification},
    error::{AppError, AppResult},
    kubectl::pods::Pod,
};

const TICK_FPS: f64 = 30.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEventResult {
    Consumed,
    Ignored,
}

pub enum AppEvent {
    Crossterm(CrosstermEvent),
    Tick,
    FocusNext,
    FocusPrev,
    Quit,
    ShowNotification(Notification),
    OpenModal(Modal),
    Explorer(ExplorerEvent),
    Logs(LogsEvent),
}

pub enum ExplorerEvent {
    Show(ExplorerKind),
    ClustersLoaded(Vec<String>),
    NamespacesLoaded {
        cluster: String,
        namespaces: Vec<String>,
    },
    PodsLoaded {
        namespace: String,
        pods: Vec<Pod>,
    },
    PodsUpdated {
        namespace: String,
        pods: Vec<Pod>,
    },
    PodLogsFinished {
        pod_name: String,
    },
}

pub enum LogsEvent {
    Load {
        namespace: String,
        pod_name: String,
    },
    Loaded {
        namespace: String,
        pod_name: String,
        logs: Option<Vec<String>>,
    },
    Reload,
    Reloaded {
        pod_name: String,
        logs: Vec<String>,
    },
}

impl From<ExplorerEvent> for AppEvent {
    fn from(value: ExplorerEvent) -> Self {
        AppEvent::Explorer(value)
    }
}

impl From<LogsEvent> for AppEvent {
    fn from(value: LogsEvent) -> Self {
        AppEvent::Logs(value)
    }
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
    pub fn send(&self, message: impl Into<AppEvent>) {
        let _ = self.sender.send(message.into());
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
        let mut reader = EventStream::new();
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
