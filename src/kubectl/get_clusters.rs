use crate::{error::AppResult, kubectl::run_kubectl_command};

pub async fn get_clusters() -> AppResult<Vec<String>> {
    let output = run_kubectl_command("kubectl", &["config", "get-clusters"]).await?;

    let lines: Vec<String> = String::from_utf8_lossy(&output)
        .lines()
        .skip(1)
        .map(|line| line.to_owned())
        .collect();

    Ok(lines)
}
