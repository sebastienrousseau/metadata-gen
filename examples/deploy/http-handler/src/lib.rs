// SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! An HTTP front end for the metadata-gen component.
//!
//! `POST /` with a Markdown document returns its front matter as a
//! JSON object (`{"title":"Hi"}`), `422` with `{"error": ...}` when it
//! has none or it does not parse, and `405` for any other method. The
//! parsing happens in the imported `metadata-gen:lib/metadata`
//! interface; this component only moves bytes and writes JSON.

use std::io::{Read as _, Write as _};

use wasip2::http::types::{
    Fields, IncomingRequest, Method, OutgoingBody, OutgoingResponse,
    ResponseOutparam,
};

wit_bindgen::generate!({
    inline: "
        package metadata-gen:http-handler;
        world handler {
            import metadata-gen:lib/metadata@0.0.10;
        }
    ",
    path: "../../../wit",
    generate_all,
});

use metadata_gen::lib::metadata;

wasip2::http::proxy::export!(Handler);

struct Handler;

impl wasip2::exports::http::incoming_handler::Guest for Handler {
    fn handle(
        request: IncomingRequest,
        response_out: ResponseOutparam,
    ) {
        let (status, body) = match request.method() {
            Method::Post => match read_body(&request) {
                Some(text) => respond(&text),
                None => (400, error_json("request body is not UTF-8")),
            },
            _ => (405, "POST a Markdown document\n".to_owned()),
        };
        send(response_out, status, &body);
    }
}

/// Runs `extract` and renders the result as the HTTP status and body.
fn respond(text: &str) -> (u16, String) {
    match metadata::extract(text) {
        Ok(entries) => (200, entries_json(&entries)),
        Err(message) => (422, error_json(&message)),
    }
}

fn read_body(request: &IncomingRequest) -> Option<String> {
    let body = request.consume().ok()?;
    let mut bytes = Vec::new();
    {
        let mut stream = body.stream().ok()?;
        stream.read_to_end(&mut bytes).ok()?;
    }
    String::from_utf8(bytes).ok()
}

fn send(response_out: ResponseOutparam, status: u16, body: &str) {
    let headers = Fields::new();
    let content_type = if status == 405 {
        "text/plain"
    } else {
        "application/json"
    };
    let _ = headers
        .set("content-type", &[content_type.as_bytes().to_vec()]);
    let response = OutgoingResponse::new(headers);
    let _ = response.set_status_code(status);
    let Ok(outgoing) = response.body() else {
        return;
    };
    ResponseOutparam::set(response_out, Ok(response));
    if let Ok(mut out) = outgoing.write() {
        let _ = out.write_all(body.as_bytes());
        let _ = out.flush();
    }
    let _ = OutgoingBody::finish(outgoing, None);
}

/// `{"key":"value",...}` in the order `extract` returns (sorted by key).
fn entries_json(entries: &[metadata::Entry]) -> String {
    let mut json = String::from("{");
    for (i, entry) in entries.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        push_string(&mut json, &entry.key);
        json.push(':');
        push_string(&mut json, &entry.value);
    }
    json.push('}');
    json
}

fn error_json(message: &str) -> String {
    let mut json = String::from("{\"error\":");
    push_string(&mut json, message);
    json.push('}');
    json
}

/// Appends `s` as a JSON string literal (RFC 8259 section 7).
fn push_string(json: &mut String, s: &str) {
    json.push('"');
    for c in s.chars() {
        match c {
            '"' => json.push_str("\\\""),
            '\\' => json.push_str("\\\\"),
            '\n' => json.push_str("\\n"),
            '\r' => json.push_str("\\r"),
            '\t' => json.push_str("\\t"),
            c if u32::from(c) < 0x20 => {
                json.push_str(&format!("\\u{:04x}", u32::from(c)));
            }
            c => json.push(c),
        }
    }
    json.push('"');
}
