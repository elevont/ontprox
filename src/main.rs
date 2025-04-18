// SPDX-FileCopyrightText: 2024 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod cache;
mod cli;
mod constants;
mod ont_request;
mod util;

use crate::ont_request::DlOrConv;
use crate::ont_request::OntRequest;
use axum::body::Body;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use cache::{annotate_ont_files, dl_ont, ont_dir, ont_file, search_ont_files};
use clap::crate_name;
use cli_utils::logging;
use cli_utils::BoxResult;
use rdfoothills_base as base;
use rdfoothills_conversion as conversion;
use rdfoothills_conversion::OntFile;
use rdfoothills_mime as mime;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use tracing_subscriber::filter::LevelFilter;
use util::{body_from_content, body_from_file, body_response, respond_with_body};

use git_version::git_version;

// This tests rust code in the README with doc-tests.
// Though, It will not appear in the generated documentation.
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

pub const VERSION: &str = git_version!(cargo_prefix = "", fallback = "unknown");

#[derive(Clone, Debug)]
pub struct Config {
    addr: SocketAddr,
    cache_root: PathBuf,
    prefer_conversion: DlOrConv,
    /// Time to wait for response when fetching the RDF source.
    /// See also [`crate::ont_request::OntRequest::timeout`].
    timeout: Duration,
}

fn main() -> BoxResult<()> {
    let log_reload_handle = logging::setup(crate_name!())?;

    let cli_args = cli::parse()?;

    let log_level = if cli_args.verbose {
        LevelFilter::DEBUG
    } else if cli_args.quiet {
        LevelFilter::WARN
    } else {
        LevelFilter::INFO
    };
    logging::set_log_level_tracing(&log_reload_handle, log_level)?;

    run_proxy(&cli_args.proxy_conf);

    Ok(())
}

#[tokio::main]
async fn run_proxy(config: &Config) {
    base::util::create_dir_async(config.cache_root.as_path()).await;

    // build our application
    let route = Router::new().route("/", get(handler_rdf).with_state(config.clone()));

    // run it
    tokio::join!(serve(route, config.addr));
}

async fn serve(app: Router, addr: SocketAddr) {
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|err| {
            let addition_opt = if addr.port() < 1024 {
                format!(" - You might need root privileges to listen on port {}, because it is smaller then 1024", addr.port())
            } else {
                String::new()
            };
            format!("Failed to listen on {addr}{addition_opt}: {err}")
        })
        .unwrap();
    tracing::info!("listening on {addr}");
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}

async fn convert(
    input_ont_file: &OntFile,
    output_ont_file: &OntFile,
    cached: bool,
) -> Result<(HeaderMap, Body), (StatusCode, String)> {
    let converter_info = conversion::convert_async(input_ont_file, output_ont_file)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "Failed to convert the {} ontology: {err}",
                    if cached { "cached" } else { "downloaded" }
                ),
            )
        })?;
    tracing::info!(
        "Converted from {} to {} using converter {}",
        input_ont_file.mime_type,
        output_ont_file.mime_type,
        converter_info.name
    );
    body_response(output_ont_file).await
}

async fn handler_rdf(
    State(config): State<Config>,
    ont_request: OntRequest,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let ont_cache_dir = ont_dir(&config.cache_root, &ont_request.uri);
    let ont_file_required = ont_file(&ont_cache_dir, ont_request.mime_type);

    let ont_might_be_cached = base::util::ensure_dir_exists_async(&ont_cache_dir)
        .await
        .map_err(|err| format!("Failed to ensure directory path exists - '{err}'"))
        .unwrap();

    if ont_might_be_cached {
        let ont_file_required_exists = base::util::look_for_file_async(&ont_file_required)
            .await
            .unwrap();
        if ont_file_required_exists {
            return Ok(respond_with_body(
                &ont_file_required,
                ont_request.mime_type,
                body_from_file(&ont_file_required).await?,
            ));
        }
    }

    // NOTE From here on we know, that the format requested by the client is not cached yet

    match ont_request.pref {
        DlOrConv::Download => {}
        DlOrConv::Convert => {
            let ont_cache_files_found = if ont_might_be_cached {
                search_ont_files(&ont_cache_dir, true).await.unwrap()
            } else {
                vec![]
            };
            if !ont_cache_files_found.is_empty() {
                let annotated_ont_cache_file_found = annotate_ont_files(ont_cache_files_found)
                    .await
                    .map_err(|err| format!("Failed to parse MIME types from cache files - '{err}'"))
                    .unwrap();
                let machine_readable_cached_ont_files: Vec<_> = annotated_ont_cache_file_found
                    .iter()
                    .filter(|ont_cache| mime::Type::is_machine_readable(ont_cache.mime_type))
                    .collect();
                for mr_ont_cache_file in machine_readable_cached_ont_files {
                    let requested_ont_file_path =
                        cache::ont_file(&ont_cache_dir, ont_request.mime_type);
                    let requested_ont_file = OntFile {
                        file: requested_ont_file_path,
                        mime_type: ont_request.mime_type,
                    };
                    if let Ok(header_body) =
                        convert(mr_ont_cache_file, &requested_ont_file, true).await
                    {
                        return Ok(header_body);
                    }
                }
            }
        }
    }

    // NOTE At this point we know that the format requested by the client
    //      is producible by converting from any of the already cached formats
    //     (if any).

    let ont_dl = dl_ont(&ont_request, &ont_cache_dir).await?;

    if ont_dl.mime_type == ont_request.mime_type {
        // This is possible if we just downloaded the ontology
        Ok(respond_with_body(
            &ont_dl.file,
            ont_request.mime_type,
            body_from_content(ont_dl.content),
        ))
    } else {
        // This is possible, if the ontology server returned a different format then the one we requested
        if ont_dl.mime_type.is_machine_readable() {
            let ont_dl_file = ont_dl.into_ont_file();
            let requested_ont_file_path = cache::ont_file(&ont_cache_dir, ont_request.mime_type);
            let requested_ont_file = OntFile {
                file: requested_ont_file_path,
                mime_type: ont_request.mime_type,
            };
            convert(&ont_dl_file, &requested_ont_file, false).await
        } else {
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!(
                "As the format returned by the server ({}) is not machine-readable, it cannot be converted into the requested format.",
                ont_dl.mime_type
            )))
        }
    }
}
