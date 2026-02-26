use crate::{
    error::{AppError, AppResult},
    kubectl::run_kubectl_command,
};

pub async fn delete_pod(namespace: &str, pod_name: &str) -> AppResult<()> {
    let command = "kubectl";
    let args = ["delete", "pod", "--namespace", namespace, pod_name];

    let output = run_kubectl_command(command, &args).await?;

    let expected_response = format!("pod \"{pod_name}\" deleted\n");
    let response = String::from_utf8_lossy(&output);

    if response.as_ref() != expected_response.as_str() {
        return Err(AppError::FailedRunKubeCtlCommand(anyhow::anyhow!(
            "Failed to delete pod {pod_name}:\n{response}"
        )));
    }

    Ok(())
}
