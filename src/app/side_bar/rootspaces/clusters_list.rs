use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::Style,
    text::Line,
};

use crate::{
    app::{
        cache::{ClusterCache, ClustersListCache},
        common::{
            FOCUS_COLOR, FilterableList, HelpMenuEnum, ListEvent, ListItemTrait, Spinner,
            handle_general_keys,
        },
        events::{AppEvent, EventSender},
        notification::Notification,
    },
    error::AppResult,
    kubectl,
};

#[derive(Debug, Clone)]
pub struct Cluster {
    name: String,
    is_selected: bool,
    spinner: Option<Spinner>,
}

#[derive(Debug, Clone)]
pub struct ClustersList {
    list: FilterableList<Cluster>,
    event_sender: EventSender,
}

impl ClustersList {
    pub fn from_cache(value: ClustersListCache, event_sender: EventSender) -> Self {
        Self {
            list: value.list.into(),
            event_sender,
        }
    }

    pub fn new(event_sender: EventSender) -> Self {
        Self {
            list: FilterableList::new(String::from("Clusters"), true),
            event_sender,
        }
    }

    pub async fn load_clusters(&mut self) -> AppResult<()> {
        let clusters = kubectl::get_clusters().await?;

        self.list.set_items(
            clusters
                .into_iter()
                .map(|item| Cluster {
                    is_selected: false,
                    name: item,
                    spinner: None,
                })
                .collect(),
        );

        Ok(())
    }

    pub fn draw<'a>(
        &'a mut self,
        area: Rect,
        frame: &mut Frame,
        is_focused: bool,
        title: impl Into<Line<'a>>,
    ) {
        self.list.draw_with_title(area, frame, is_focused, title);
    }

    async fn on_cluster_selected(&mut self, cluster: Cluster) {
        let cluster_ref = self
            .list
            .inner_list
            .iter_mut()
            .find(|item| item.name == cluster.name)
            .unwrap();

        let spinner = Spinner::new();

        cluster_ref.spinner = Some(spinner.clone());

        let cluster_name = cluster.name.clone();
        let event_sender = self.event_sender.clone();

        tokio::spawn(async {
            Self::load_namespaces(cluster_name, spinner, event_sender).await;
        });
    }

    async fn load_namespaces(
        cluster_name: String,
        mut spinner: Spinner,
        event_sender: EventSender,
    ) {
        let namespaces = match kubectl::get_namespaces(&cluster_name).await {
            Ok(data) => data,
            Err(err) => {
                event_sender.send(AppEvent::ShowNotification(Notification::error(err)));
                return;
            }
        };

        spinner.stop();

        event_sender.send(AppEvent::LoadNamespaces(namespaces));
    }

    pub async fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if let Some(list_event) = self.list.handle_key(key) {
            match list_event {
                ListEvent::Quit => {
                    self.event_sender.send(AppEvent::Quit);
                }
                ListEvent::SelectedItem(cluster) => self.on_cluster_selected(cluster).await,
                ListEvent::StayInList => {}
            };
            return true;
        }

        match key.code {
            KeyCode::Char('r') => {
                if let Err(err) = self.load_clusters().await {
                    self.event_sender
                        .send(AppEvent::ShowNotification(Notification::error(err)));
                }

                return true;
            }
            KeyCode::Char('?') => {
                self.event_sender
                    .send(AppEvent::ShowHelp(HelpMenuEnum::Clusters));
            }
            _ => {}
        }

        handle_general_keys(key, &self.event_sender)
    }
}

impl From<ClusterCache> for Cluster {
    fn from(value: ClusterCache) -> Self {
        Self {
            is_selected: value.is_selected,
            name: value.name,
            spinner: None,
        }
    }
}

impl From<Cluster> for ClusterCache {
    fn from(value: Cluster) -> Self {
        Self {
            is_selected: value.is_selected,
            name: value.name,
        }
    }
}

impl ListItemTrait for Cluster {
    fn get_style(&self) -> Option<Style> {
        if self.is_selected {
            return Some(FOCUS_COLOR.into());
        }

        None
    }

    fn as_ref(&self) -> &str {
        &self.name
    }

    fn spinner(&self) -> Option<String> {
        self.spinner
            .as_ref()
            .and_then(|spinner| spinner.get_spin_state().map(|spinner| spinner.to_string()))
    }
}

impl From<ClustersList> for ClustersListCache {
    fn from(value: ClustersList) -> Self {
        Self {
            list: value.list.into(),
        }
    }
}
