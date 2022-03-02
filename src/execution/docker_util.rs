use crate::error::Error::ExecutionTimeout;
use futures_util::stream::StreamExt;
use shiplift::rep::Exit;
use shiplift::tty::TtyChunk;
use shiplift::{Container, ContainerOptions, Docker, LogsOptions, RmContainerOptions};
use std::time::Duration;

async fn start_and_wait_container(
    container: Container<'_>,
    timeout: Duration,
) -> Result<(Exit, String, String), anyhow::Error> {
    log::trace!(
        "Starting container {} and waiting for its completion",
        container.id()
    );

    container.start().await?;

    let res = async_std::future::timeout(timeout, container.wait()).await;

    let res = match res {
        Ok(r) => r,
        Err(_) => return Err(ExecutionTimeout.into()),
    };

    let res = res?;

    let mut stream = container.logs(&LogsOptions::builder().stderr(true).stdout(true).build());

    let mut stderr: Vec<u8> = Vec::new();
    let mut stdout: Vec<u8> = Vec::new();
    while let Some(message) = stream.next().await {
        let message: TtyChunk = message?;
        match message {
            TtyChunk::StdIn(_) => {}
            TtyChunk::StdOut(mut m) => stdout.append(&mut m),
            TtyChunk::StdErr(mut m) => stderr.append(&mut m),
        }
    }

    let stderr = std::str::from_utf8(&stderr)?.to_string();
    let stdout = std::str::from_utf8(&stdout)?.to_string();

    Ok((res, stdout, stderr))
}

pub async fn run_container(
    docker: &Docker,
    container_options: &ContainerOptions,
    timeout: Duration,
) -> Result<(Exit, String, String), anyhow::Error> {
    let container = docker.containers().create(container_options).await?;

    let wait_res =
        start_and_wait_container(docker.containers().get(container.id.clone()), timeout).await;

    // no matter the wait_res - remove the container
    docker
        .containers()
        .get(container.id)
        .remove(RmContainerOptions::builder().force(true).build())
        .await?;

    Ok(wait_res?)
}
