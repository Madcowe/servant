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

        if url.host_str() == Some("settings") {
            if url.path() == "/clear-cache" {
                self.resolver.clear_cache();
            }
            let response = self.settings_ui.handle_request(request);
            return Box::pin(std::future::ready(response));
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
