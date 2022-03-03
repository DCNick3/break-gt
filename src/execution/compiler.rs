use crate::error::Error::CompilationError;
use crate::execution::docker_util::run_container;
use futures_util::stream::StreamExt;
use shiplift::{ContainerOptions, Docker, PullOptions};
use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::time::Duration;
use tempfile::{tempdir, TempDir};
use tracing::{debug, info, instrument, trace};

pub struct JavaCompiler {
    docker: Docker,
    image_name: String,
}

impl Debug for JavaCompiler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JavaCompiler")
            .field("image_name", &self.image_name)
            .finish()
    }
}

const IMAGE_NAME: &str = "openjdk:8-alpine";
const TIMEOUT: Duration = Duration::from_secs(5);

impl JavaCompiler {
    pub async fn new(docker: Docker) -> Result<JavaCompiler, anyhow::Error> {
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

    #[instrument(skip(program))]
    pub async fn compile(
        &self,
        program: &JavaProgram,
    ) -> Result<CompiledJavaProgram, anyhow::Error> {
        let dir = tempdir()?;

        trace!("Compiling java program in {dir:?}");

        let mut java_paths = Vec::new();

        for JavaClass {
            full_name,
            source_code,
        } in program.0.iter()
        {
            let path = full_name.replace('.', "/") + ".java";
            let path = dir.path().join(path);

            std::fs::create_dir_all(path.parent().unwrap())?;
            std::fs::write(path.clone(), source_code)?;

            java_paths.push(path);
        }

        let mounts = vec![format!("{}:/app", dir.path().to_str().unwrap())];
        let mut cmd: Vec<String> = ["javac", "-sourcepath", "/app"]
            .iter()
            .map(|s| s.to_string())
            .collect();

        cmd.extend(
            program
                .0
                .iter()
                .map(|p| format!("/app/{}.java", p.full_name.replace('.', "/"))),
        );

        trace!("Creating compiler container...");
        let container = ContainerOptions::builder(&self.image_name)
            .volumes(mounts.iter().map(|s| s.as_str()).collect())
            .cmd(cmd.iter().map(|s| s.as_str()).collect())
            .network_mode("none")
            // .attach_stderr(true)
            // .attach_stdout(true)
            .build();

        let (exit, _, err) = run_container(&self.docker, &container, TIMEOUT).await?;

        if exit.status_code != 0 {
            return Err(CompilationError(err).into());
        }

        trace!("javac succeeded, removing source code");
        for class in java_paths {
            std::fs::remove_file(class)?
        }

        Ok(CompiledJavaProgram { directory: dir })
    }
}

#[derive(Debug)]
pub struct JavaClass {
    pub full_name: String,
    pub source_code: String,
}

#[derive(Debug)]
pub struct JavaProgram(Vec<JavaClass>);

impl JavaProgram {
    pub fn new() -> Self {
        JavaProgram(Vec::new())
    }

    pub fn push_class(&mut self, full_name: String, source_code: String) {
        self.0.push(JavaClass {
            full_name: full_name.to_string(),
            source_code,
        })
    }
}

#[derive(Debug)]
pub struct CompiledJavaProgram {
    directory: TempDir,
}

impl CompiledJavaProgram {
    pub fn path(&self) -> &Path {
        self.directory.path()
    }
}
