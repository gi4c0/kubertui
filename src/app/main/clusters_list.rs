use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::Style,
};

use crate::{
    app::{
        cache::{ClusterCache, ClustersListCache},
        common::{FOCUS_COLOR, FilterableList, HelpMenuEnum, ListEvent, ListItemTrait, Spinner},
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
            list: FilterableList::new(String::from("Clusters")).filterable(),
            event_sender,
        }
    }

    pub async fn load_clusters(&mut self) -> AppResult<()> {
        let clusters = kubectl::get_clusters().await?;
        self.set_clusters(clusters);

        Ok(())
    }

    pub fn set_clusters(&mut self, clusters: Vec<String>) {
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
    }

    pub fn draw(&mut self, area: Rect, frame: &mut Frame) {
        self.list.draw_with_title(area, frame, "Clusters");
    }

    fn on_cluster_selected(&mut self, cluster: Cluster) {
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

    pub fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        if let Some(list_event) = self.list.handle_key(key) {
            match list_event {
                ListEvent::Quit => {
                    self.event_sender.send(AppEvent::Quit);
                }
                ListEvent::SelectedItem(cluster) => self.on_cluster_selected(cluster),
                ListEvent::StayInList => {}
            };
            return true;
        }

        match key.code {
            KeyCode::Char('r') => {
                let event_sender = self.event_sender.clone();

                tokio::spawn(async move {
                    match kubectl::get_clusters().await {
                        Ok(clusters) => event_sender.send(AppEvent::ClustersLoaded(clusters)),
                        Err(err) => event_sender
                            .send(AppEvent::ShowNotification(Notification::error(err))),
                    }
                });

                true
            }

            KeyCode::Char('?') => {
                self.event_sender
                    .send(AppEvent::ShowHelp(HelpMenuEnum::Clusters));

                true
            }
            _ => false,
        }
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
