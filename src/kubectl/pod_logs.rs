use crate::{error::AppResult, kubectl::run_kubectl_command};

pub async fn load_logs(namespace: &str, pod_name: &str) -> AppResult<Vec<String>> {
    let output = run_kubectl_command("kubectl", &["-n", namespace, "logs", pod_name]).await?;

    let result: Vec<String> = String::from_utf8_lossy(&output)
        .lines()
        .map(|line| line.to_owned())
        .collect::<Vec<_>>();

    Ok(result)
}
