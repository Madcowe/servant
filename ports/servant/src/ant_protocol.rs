use std::future::Future;
use std::pin::Pin;

use http::header::HeaderValue;
use net_traits::http_status::HttpStatus;
use net_traits::request::Request;
use net_traits::response::{Response, ResponseBody};
use net_traits::{NetworkError, ResourceFetchTiming};
use net::protocols::ProtocolHandler;
use net::fetch::methods::{DoneChannel, FetchContext};

use crate::ant_url::AntUrl;
use crate::content_resolver::{ContentResolver, ResolvedContent};
use crate::settings::SettingsUi;
use crate::ant_client::AntClientManager;
use crate::loading_html::LOADING_HTML;
use crate::loading::LoadingTracker;
use std::sync::Arc;

pub struct AntProtocolHandler {
    resolver: ContentResolver,
    settings_ui: SettingsUi,
    _manager: Arc<AntClientManager>,
}

impl AntProtocolHandler {
    pub fn new(resolver: ContentResolver, manager: Arc<AntClientManager>) -> Self {
        Self { 
            resolver,
            settings_ui: SettingsUi::new(),
            _manager: manager,
        }
    }
}

impl ProtocolHandler for AntProtocolHandler {
    fn load<'a>(
        &'a self,
        request: &'a mut Request,
        _done_chan: &mut DoneChannel,
        _context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send + 'a>> {
        let url = request.current_url();
        let timing_type = request.timing_type();

        if url.as_url().host_str() == Some("settings") {
            if url.as_url().path() == "/clear-cache" {
                self.resolver.clear_cache();
            }
            let response = self.settings_ui.handle_request(request);
            return Box::pin(std::future::ready(response));
        }

        if url.as_url().host_str() == Some("loading-status") {
            let address = url.as_url().path().trim_start_matches('/');
            if let Some(progress) = LoadingTracker::get_progress(address) {
                let json = format!(
                    r#"{{"status": "{}", "bytes_loaded": {}, "total_bytes": {}, "error": {}, "finished": {}}}"#,
                    progress.status,
                    progress.bytes_loaded,
                    progress.total_bytes.map(|b| b.to_string()).unwrap_or_else(|| "null".to_string()),
                    progress.error.as_ref().map(|e| format!("\"{}\"", e)).unwrap_or_else(|| "null".to_string()),
                    progress.finished
                );
                let mut response = Response::new(url, ResourceFetchTiming::new(timing_type));
                *response.body.lock() = ResponseBody::Done(json.into_bytes());
                if let Ok(hv) = HeaderValue::from_str("application/json") {
                    response.headers.insert(http::header::CONTENT_TYPE, hv);
                }
                return Box::pin(std::future::ready(response));
            } else {
                let json = r#"{"status": "Error", "bytes_loaded": 0, "total_bytes": null, "error": "No progress found for this address", "finished": true}"#;
                let mut response = Response::new(url, ResourceFetchTiming::new(timing_type));
                *response.body.lock() = ResponseBody::Done(json.as_bytes().to_vec());
                if let Ok(hv) = HeaderValue::from_str("application/json") {
                    response.headers.insert(http::header::CONTENT_TYPE, hv);
                }
                return Box::pin(std::future::ready(response));
            }
        }

        Box::pin(async move {
            let ant_url = match AntUrl::parse(url.as_url()) {
                Ok(u) => u,
                Err(e) => {
                    return Response::network_error(NetworkError::ResourceLoadError(
                        format!("Invalid ant:// URL: {:?}", e)
                    ));
                }
            };

            let query: std::collections::HashMap<_, _> = url.as_url().query_pairs().collect();
            let is_raw = query.contains_key("servant_raw");

            // If it's a new address and not raw, show the loading page
            if !is_raw && !self.resolver.is_cached(&ant_url.address) {
                let resolver = self.resolver.clone();
                let addr = ant_url.address;
                let sub_path_opt = ant_url.sub_path.clone();
                
                tokio::spawn(async move {
                    let sub_path = sub_path_opt.as_deref();
                    let _ = resolver.resolve(&addr, sub_path).await;
                });

                let mut response = Response::new(url, ResourceFetchTiming::new(timing_type));
                *response.body.lock() = ResponseBody::Done(LOADING_HTML.as_bytes().to_vec());
                if let Ok(hv) = HeaderValue::from_str("text/html") {
                    response.headers.insert(http::header::CONTENT_TYPE, hv);
                }
                return response;
            }

            let sub_path = ant_url.sub_path.as_deref();
            match self.resolver.resolve(&ant_url.address, sub_path).await {
                Ok(ResolvedContent::SingleFile { data, mime }) | 
                Ok(ResolvedContent::RawChunk { data, mime }) => {
                    let mut response = Response::new(url, ResourceFetchTiming::new(timing_type));
                    *response.body.lock() = ResponseBody::Done(data.to_vec());
                    if let Ok(hv) = HeaderValue::from_str(&mime) {
                        response.headers.insert(http::header::CONTENT_TYPE, hv);
                    }
                    response.status = HttpStatus::default();
                    response
                }
                Err(e) => {
                    Response::network_error(NetworkError::ResourceLoadError(
                        format!("Resolution failed: {}", e)
                    ))
                }
            }
        })
    }

    fn is_fetchable(&self) -> bool { true }
    fn is_secure(&self) -> bool { true }
}
