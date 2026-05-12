/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::{ResourceFetchTiming};
use net_traits::http_status::HttpStatus;
use http::header::HeaderValue;

#[derive(Clone)]
pub struct SettingsUi {}

impl SettingsUi {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_request(&self, request: &Request) -> Response {
        let url = request.current_url();
        let timing = ResourceFetchTiming::new(request.timing_type());
        let mut response = Response::new(url.clone(), timing);
        
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Servant Settings</title>
            <style>
                body { font-family: system-ui, sans-serif; max-width: 800px; margin: 40px auto; padding: 0 20px; }
                .card { border: 1px solid #ccc; border-radius: 8px; padding: 20px; margin-bottom: 20px; }
                h1 { color: #333; }
                button { background: #0066cc; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer; }
                button:hover { background: #0052a3; }
            </style>
        </head>
        <body>
            <h1>Servant Settings</h1>
            
            <div class="card">
                <h2>Network Configuration</h2>
                <p>Current Network: <strong>Mainnet</strong> (Default)</p>
                <p>To use a devnet, launch servant with: <code>--devnet-manifest /path/to/devnet.json</code></p>
                <p>To use custom bootstrap peers, launch with: <code>--bootstrap-peers addr1,addr2</code></p>
            </div>
            
            <div class="card">
                <h2>Cache</h2>
                <p>Content caching is enabled to improve performance on the Autonomi network.</p>
                <button onclick="window.location.href='ant://settings/clear-cache'">Clear Cache</button>
            </div>
        </body>
        </html>
        "#;

        *response.body.lock() = ResponseBody::Done(html.as_bytes().to_vec());
        response.headers.insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        response.status = HttpStatus::default();
        response
    }
}
