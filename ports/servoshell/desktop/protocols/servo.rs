/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Loads resources using a mapping from well-known shortcuts to resource: urls.
//! Recognized shortcuts:
//! - servo:default-user-agent
//! - servo:experimental-preferences
//! - servo:config
//! - servo:newtab
//! - servo:avomeri
//! - servo:pulloposti
//! - servo:blocked
//! - servo:locale
//! - servo:preferences

use std::future::Future;
use std::pin::Pin;

use headers::{ContentType, HeaderMapExt};
use servo::UserAgentPlatform;
use servo::protocol_handler::{
    DoneChannel, FetchContext, NetworkError, ProtocolHandler, Request, ResourceFetchTiming,
    Response, ResponseBody,
};

#[cfg(feature = "kotisatama")]
use kotisatama_i18n::parse_locale_choice;

use crate::desktop::protocols::resource::ResourceProtocolHandler;
use crate::prefs::EXPERIMENTAL_PREFS;

#[derive(Default)]
pub struct ServoProtocolHandler {}

impl ProtocolHandler for ServoProtocolHandler {
    fn privileged_paths(&self) -> &'static [&'static str] {
        &["config", "preferences"]
    }

    fn is_fetchable(&self) -> bool {
        true
    }

    fn load(
        &self,
        request: &mut Request,
        done_chan: &mut DoneChannel,
        context: &FetchContext,
    ) -> Pin<Box<dyn Future<Output = Response> + Send>> {
        let url = request.current_url();

        match url.path() {
            "config" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/config.html",
            ),
            "newtab" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/newtab.html",
            ),
            // KOTISATAMA-PATCH: sisäiset avomeri- ja pulloposti-sivut (resource_protocol).
            "avomeri" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/avomeri.html",
            ),

            "pulloposti" | "pulloposti/app" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/pulloposti.html",
            ),

            // KOTISATAMA-PATCH: whitelist-blokkaussivu (i18n: blocked.html + kotisatama-i18n.js).
            "blocked" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/blocked.html",
            ),

            // KOTISATAMA-PATCH: tallenna kielivalinta desktop-UI:lle ja ohjaa takaisin configiin.
            #[cfg(feature = "kotisatama")]
            "locale" => {
                if let Some(set) = url
                    .query_pairs()
                    .find(|(key, _)| key == "set")
                    .map(|(_, value)| value.into_owned())
                {
                    if let Some(choice) = parse_locale_choice(&set) {
                        let _ = kotisatama_i18n::set_locale_choice(choice);
                    }
                }
                html_response(
                    request,
                    "<!DOCTYPE html><html><head><meta charset=\"utf-8\">\
                     <script>location.replace('servo:config');</script></head><body></body></html>"
                        .to_owned(),
                )
            },

            "preferences" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/preferences.html",
            ),

            "license" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/license.html",
            ),

            "experimental-preferences" => {
                let pref_list = EXPERIMENTAL_PREFS
                    .iter()
                    .map(|pref| format!("\"{pref}\""))
                    .collect::<Vec<String>>()
                    .join(",");
                json_response(request, format!("[{pref_list}]"))
            },

            "default-user-agent" => {
                let user_agent = UserAgentPlatform::default().to_user_agent_string();
                json_response(request, format!("\"{user_agent}\""))
            },

            _ => Box::pin(std::future::ready(Response::network_error(
                NetworkError::ResourceLoadError("Invalid shortcut".to_owned()),
            ))),
        }
    }
}

fn json_response(
    request: &Request,
    body: String,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    let mut response = Response::new(
        request.current_url(),
        ResourceFetchTiming::new(request.timing_type()),
    );
    response.headers.typed_insert(ContentType::json());
    *response.body.lock() = ResponseBody::Done(body.into_bytes());
    Box::pin(std::future::ready(response))
}

fn html_response(
    request: &Request,
    body: String,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    let mut response = Response::new(
        request.current_url(),
        ResourceFetchTiming::new(request.timing_type()),
    );
    response.headers.typed_insert(ContentType::from(
        mime_guess::from_path("index.html").first_or_octet_stream(),
    ));
    *response.body.lock() = ResponseBody::Done(body.into_bytes());
    Box::pin(std::future::ready(response))
}
