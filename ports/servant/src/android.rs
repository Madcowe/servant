/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use jni::JavaVM;
use jni::sys::jint;
use std::os::raw::c_void;
use std::sync::Arc;
use crate::ant_client::AntClientManager;
use crate::content_resolver::ContentResolver;
use crate::ant_protocol::AntProtocolHandler;
use crate::cache::ContentCache;
use log::{info, error};

#[unsafe(no_mangle)]
pub extern "C" fn JNI_OnLoad(_vm: JavaVM, _reserved: *mut c_void) -> jint {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("servant"),
    );

    info!("JNI_OnLoad called for Servant");

    servoshell::set_protocol_registry_callback(Box::new(|protocols| {
        info!("Registering ant:// protocol on Android");
        
        // Initialize Autonomi client with default settings for Android.
        // In the future, these could be passed via intent extras or other means.
        let ant_manager = match AntClientManager::connect(None, None) {
            Ok(manager) => Arc::new(manager),
            Err(e) => {
                error!("Failed to connect to Autonomi network: {:?}", e);
                return;
            }
        };

        let cache = Arc::new(ContentCache::new(100));
        let resolver = ContentResolver::new(ant_manager.client(), cache);

        // Register ant:// protocol
        if let Err(e) = protocols.register("ant", AntProtocolHandler::new(resolver.clone(), ant_manager)) {
            error!("Failed to register ant:// protocol handler: {:?}", e);
            return;
        }

        // Provide a way for the UI to save files from the cache
        let resolver_clone = resolver.clone();
        servoshell::set_resource_data_provider(Box::new(move |url| {
            resolver_clone.get_cached_content_for_url(url).map(|(b, m)| (b.to_vec(), m))
        }));

        info!("ant:// protocol successfully registered on Android.");
    }));

    jni::sys::JNI_VERSION_1_6
}
