use crate::execution::compiler::CompiledJavaProgram;
use crate::execution::docker_util::run_container;
use futures_util::stream::StreamExt;
use shiplift::rep::Exit;
use shiplift::{ContainerOptions, Docker, PullOptions};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::time::Duration;
use tracing::{debug, info, instrument, trace};

const IMAGE_NAME: &str = "openjdk:8-alpine";
const TIMEOUT: Duration = Duration::from_secs(1);

pub struct Runner {
    docker: Docker,
    image_name: String,
}

impl Debug for Runner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runner")
            .field("image_name", &self.image_name)
            .finish()
    }
}

impl Runner {
    pub async fn new(docker: Docker) -> Result<Self, Box<dyn Error>> {
        info!("Gonna pull image {IMAGE_NAME}");
        {
            let mut stream = docker
                .images()
                .pull(&PullOptions::builder().image(IMAGE_NAME).build());

            while let Some(pull_result) = stream.next().await {
                debug!("Pull message: {}", pull_result?);
            }
        }

        Ok(Self {
            docker,
            image_name: IMAGE_NAME.to_string(),
        })
    }

    #[instrument]
    pub async fn run_java(
        &self,
        program: &CompiledJavaProgram,
        main_class: &str,
    ) -> Result<(Exit, String, String), anyhow::Error> {
        info!(
            "Running java program {:?} with class {}",
            program.path(),
            main_class
        );

        let mounts = vec![format!("{}:/app", program.path().to_str().unwrap())];
        let cmd = vec!["java", "-cp", "/app", main_class];

        trace!("Creating runner container...");
        let container = ContainerOptions::builder(&self.image_name)
            .volumes(mounts.iter().map(|s| s.as_str()).collect())
            .cmd(cmd)
            .build();

        run_container(&self.docker, &container, TIMEOUT).await
    }
}
