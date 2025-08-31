#[derive(Copy, Clone)]
pub enum RunMode {
    Binary,
    DockerSocketMounted { compose_contents: &'static str },
    DockerSocketProxy { compose_contents: &'static str },
}
