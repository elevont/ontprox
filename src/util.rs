// SPDX-FileCopyrightText: 2024 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    body::Body,
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
};
use rdfoothills_conversion::OntFile;
use rdfoothills_mime as mime;
use reqwest::header::CONTENT_DISPOSITION;
use std::path::Path as StdPath;
use tokio_util::io::ReaderStream;

pub async fn body_response(ont_file: &OntFile) -> Result<(HeaderMap, Body), (StatusCode, String)> {
    let body = body_from_file(&ont_file.file).await?;

    Ok(respond_with_body(&ont_file.file, ont_file.mime_type, body))
}

pub fn respond_with_body(file: &StdPath, mime_type: mime::Type, body: Body) -> (HeaderMap, Body) {
    let mut headers = HeaderMap::new();

    // headers.insert(CONTENT_TYPE, "text/toml; charset=utf-8".parse().unwrap());
    headers.insert(CONTENT_TYPE, mime_type.mime_type().parse().unwrap());
    let attachment = if matches!(mime_type, mime::Type::Html) {
        ""
    } else {
        "attachment; "
    };
    headers.insert(
        CONTENT_DISPOSITION,
        format!(
            "{attachment}filename=\"{}\"",
            file.file_name().unwrap().to_string_lossy()
        )
        .parse()
        .unwrap(),
    );

    (headers, body)
}

pub async fn body_from_file(file: &StdPath) -> Result<Body, (StatusCode, String)> {
    // `File` implements `AsyncRead`
    let file_handl = match tokio::fs::File::open(file).await {
        Ok(file_handl) => file_handl,
        Err(err) => {
            return Err((
                StatusCode::NOT_FOUND,
                format!("File '{}' not found: {err}", file.display()),
            ))
        }
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = ReaderStream::new(file_handl);
    // convert the `Stream` into an `axum::body::HttpBody`
    Ok(Body::from_stream(stream))
}

pub fn body_from_content(ont_content: Vec<u8>) -> Body {
    Body::from(ont_content)
}
