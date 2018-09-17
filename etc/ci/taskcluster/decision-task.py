# coding: utf8

import os.path
from decisionlib import DecisionTask


# https://docs.taskcluster.net/docs/reference/workers/docker-worker/docs/caches
CARGO_CACHE_SCOPES = [
    "docker-worker:cache:cargo-registry-cache",
    "docker-worker:cache:cargo-git-cache",
]

CARGO_CACHE = {
    "cargo-registry-cache": "/root/.cargo/registry",
    "cargo-git-cache": "/root/.cargo/git",
}

BUILD_ENV = {
    "RUST_BACKTRACE": "1",
    "RUSTFLAGS": "-Dwarnings",
    "CARGO_INCREMENTAL": "0",
    "SCCACHE_IDLE_TIMEOUT": "1200",
}


def main():
    decision = DecisionTask(
        project_name="Servo",  # Used in task names
        route_prefix="project.servo.servo",
        docker_image_cache_expiry="1 week",
        worker_type="servo-docker-worker",
    )

    decision.create_task_with_in_tree_dockerfile(
        task_name="building for Linux x86_64 in dev mode",
        command="""
            ./mach build --dev
        """,
        env=BUILD_ENV,
        dockerfile=dockerfile("build-x86_64-linux"),
        max_run_time_minutes=3 * 60,
        scopes=CARGO_CACHE_SCOPES,
        cache=CARGO_CACHE,
    )

    decision.create_task_with_in_tree_dockerfile(
        task_name="tidy",
        command="""
            ./mach test-tidy --no-progress --all
            ./mach test-tidy --no-progress --self-test
        """,
        dockerfile=dockerfile("build-x86_64-linux"),
        max_run_time_minutes=3 * 60,
        scopes=CARGO_CACHE_SCOPES,
        cache=CARGO_CACHE,
    )


def dockerfile(name):
    return os.path.join(os.path.dirname(__file__), name + ".dockerfile")


if __name__ == "__main__":
    main()
