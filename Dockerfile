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

# Then prepare the python dependencies
FROM python:3.13-slim-bookworm AS python-builder

RUN mkdir /src
WORKDIR /src

# Downloads rdf-tools sources
# NOTE Doing it this weay, we do not have to `apt update` and install `git`.
SHELL ["/bin/bash", "-c"]
RUN echo $'import urllib.request\nurllib.request.urlretrieve("https://github.com/elevont/rdftools/archive/refs/heads/master.zip", "rdftools.zip")\n' > download.py
RUN echo $'import zipfile\nwith zipfile.ZipFile("rdftools.zip", "r") as zip_ref:\n    zip_ref.extractall("./")\n' > extract.py
RUN python download.py
RUN python extract.py && find .

RUN mkdir /install
WORKDIR /install

RUN PYTHONUSERBASE=/install \
    pip install \
    --user \
    --upgrade \
    pylode \
    "file:///src/rdftools-master#egg=rdftools"

# Then use a minimal container
# and only copy over the required files
# generated in the previous container(s).
FROM bitnami/minideb:bookworm

RUN install_packages \
    ca-certificates

COPY --from=rust-builder /usr/local/cargo/bin/* /usr/local/bin/
COPY --from=python-builder /install /usr/local

# NOTE Labels and annotaitons are added by CI (outside this Dockerfile);
#      see `.github/workflows/docker.yml`.
#      This also means they will not be available in local builds.

ENTRYPOINT ["ontprox"]
CMD ["--address", "0.0.0.0", "--port", "80"]
EXPOSE 80
