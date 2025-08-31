use crate::common::run_mode::RunMode;
use crate::common::test_metrics;

mod common;

#[ignore]
#[tokio::test]
async fn binary() {
    test_metrics(RunMode::Binary).await;
}

#[ignore]
#[tokio::test]
async fn mounted_socket() {
    const COMPOSE_CONTENTS: &str = include_str!("../../examples/compose.mounted.yml");
    const RUN_MODE: RunMode = RunMode::DockerSocketMounted {
        compose_contents: COMPOSE_CONTENTS,
    };

    test_metrics(RUN_MODE).await;
}

#[ignore]
#[tokio::test]
async fn calum4_docker_socket_proxy() {
    const COMPOSE_CONTENTS: &str =
        include_str!("../../examples/compose.calum4.docker-socket-proxy.yml");
    const RUN_MODE: RunMode = RunMode::DockerSocketProxy {
        compose_contents: COMPOSE_CONTENTS,
    };

    test_metrics(RUN_MODE).await;
}

#[ignore]
#[tokio::test]
async fn wollomatic_socket_proxy() {
    const COMPOSE_CONTENTS: &str =
        include_str!("../../examples/compose.wollomatic.socket-proxy.yml");
    const RUN_MODE: RunMode = RunMode::DockerSocketProxy {
        compose_contents: COMPOSE_CONTENTS,
    };

    test_metrics(RUN_MODE).await;
}

#[ignore]
#[tokio::test]
async fn linuxserver_docker_socket_proxy() {
    const COMPOSE_CONTENTS: &str =
        include_str!("../../examples/compose.linuxserver.docker-socket-proxy.yml");
    const RUN_MODE: RunMode = RunMode::DockerSocketProxy {
        compose_contents: COMPOSE_CONTENTS,
    };

    test_metrics(RUN_MODE).await;
}
