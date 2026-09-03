use ratatui::widgets::Cell;

use crate::{
    app::{
        events::{AppEvent, EventSender},
        notification::Notification,
    },
    kubectl::{
        self,
        pods::{KnownPodStatus, PodStatus, get_pods_list},
    },
};

pub fn get_status<'a>(statuses: &'a [PodStatus], reason: &'a Option<String>) -> Cell<'a> {
    if statuses.len() <= 5 {
        let statuses: Vec<String> = statuses
            .iter()
            .map(|status| match status {
                PodStatus::Unknown(_) => "❓".into(),

                PodStatus::Known(known_status) => match known_status {
                    KnownPodStatus::Running { started_at: _ } => "💚".into(),
                    KnownPodStatus::Terminated {
                        container_id: _,
                        exit_code: _,
                        finished_at: _,
                        reason: _,
                        started_at: _,
                        message: _,
                    } => "💔".into(),
                    KnownPodStatus::Waiting {
                        reason: _,
                        message: _,
                    } => "💤".into(),
                },
            })
            .collect();

        if statuses.is_empty()
            && let Some(reason) = reason
        {
            let icon = match reason.as_str() {
                "Evicted" => "❌".to_string(),
                another => format!("❌ ({another})").to_string(),
            };

            return Cell::from(icon);
        }

        return Cell::from(statuses.join(" "));
    }

    let running = statuses
        .iter()
        .filter(|status| {
            matches!(
                status,
                PodStatus::Known(KnownPodStatus::Running { started_at: _ })
            )
        })
        .count();

    Cell::from(format!("{}/{}", running, statuses.len()))
}

pub fn delete_pod(namespace: String, pod_name: String, event_sender: EventSender) {
    tokio::spawn(async move {
        if let Err(err) = kubectl::delete_pod(&namespace, &pod_name).await {
            event_sender.send(AppEvent::ShowNotification(Notification::error(err)));
        }

        match get_pods_list(namespace.as_str()).await {
            Ok(pods) => event_sender.send(AppEvent::PodsUpdated { namespace, pods }),
            Err(err) => event_sender.send(AppEvent::ShowNotification(Notification::error(err))),
        }
    });
}
