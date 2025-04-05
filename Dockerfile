# syntax=docker/dockerfile:1
# NOTE Lint this file with https://hadolint.github.io/hadolint/

# SPDX-FileCopyrightText: 2022-2024 Robin Vobruba <hoijui.quaero@gmail.com>
#
# SPDX-License-Identifier: Unlicense

# First, compile in the rust container
FROM rust:1.82-bookworm AS rust-builder
WORKDIR /usr/src/app
COPY ["Cargo.*", "."]
COPY ["src", "./src"]
RUN cargo install --path .

# Then use a minimal container
# and only copy over the required files
# generated in the previous container(s).
FROM bitnami/python:3.13-debian-12

RUN install_packages \
    ca-certificates \
    default-jre-headless

WORKDIR /tmp/work

COPY requirements.txt ./
RUN \
    pip install \
        --no-cache-dir \
        --user \
        --upgrade \
        --requirement requirements.txt && \
    rm requirements.txt
ENV PATH="$PATH:/root/.local/bin/"

COPY --from=rust-builder /usr/local/cargo/bin/* /usr/local/bin/

WORKDIR /root/.local/bin/

COPY ["run/downloader", "./"]

# Setup ROBOT (RDF tool; also supports conversion; written in Java)
RUN \
    downloader \
        "https://github.com/ontodev/robot/releases/download/v1.9.7/robot.jar" \
    && downloader \
        "https://raw.githubusercontent.com/ontodev/robot/master/bin/robot" \
    && chmod +x robot

# NOTE Labels and annotations are added by CI (outside this Dockerfile);
#      see `.github/workflows/docker.yml`.
#      This also means they will not be available in local builds.

ENTRYPOINT ["ontprox"]
CMD ["--address", "0.0.0.0", "--port", "80"]
EXPOSE 80
