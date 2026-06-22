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
//! - servo:whitelist
//! - servo:whitelist/add
//! - servo:whitelist/commit-add
//! - servo:whitelist/remove
//! - servo:whitelist/commit-remove
//! - servo:whitelist/list
//! - servo:blocked
//! - servo:locale
//! - servo:preferences

#[cfg(feature = "kotisatama")]
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
#[cfg(feature = "kotisatama")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "kotisatama")]
use std::sync::{Mutex, OnceLock};

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

            "pulloposti" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/pulloposti.html",
            ),

            "pulloposti/app" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/pulloposti-app.html",
            ),

            // KOTISATAMA-PATCH: whitelist-blokkaussivu (i18n: blocked.html + kotisatama-i18n.js).
            "blocked" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/blocked.html",
            ),

            // KOTISATAMA-PATCH: käyttäjän omat whitelist-sivut (overlay, ei CDN:ää).
            #[cfg(feature = "kotisatama")]
            "whitelist/add" => {
                use kotisatama_whitelist::normalize_domain;

                let domain = query_param(url.as_url(), "domain");
                let return_url = query_param(url.as_url(), "return");
                let domain = match domain.and_then(|domain| normalize_domain(&domain).ok()) {
                    Some(domain) => domain,
                    None => return redirect_response(request, "servo:whitelist?error=invalid"),
                };
                let token = register_pending_whitelist_change("add", domain.clone(), return_url);
                whitelist_confirm_response(request, "add", &domain, &token)
            },

            #[cfg(feature = "kotisatama")]
            "whitelist/commit-add" => {
                use kotisatama_whitelist::{add_user_domain, is_navigation_allowed};
                use url::Url;

                let domain = query_param(url.as_url(), "domain");
                let token = query_param(url.as_url(), "token");
                let pending = match (domain, token) {
                    (Some(domain), Some(token)) => {
                        take_pending_whitelist_change("add", &domain, &token)
                    },
                    _ => None,
                };
                let Some(pending) = pending else {
                    return redirect_response(request, "servo:whitelist?error=failed");
                };
                let redirect = match add_user_domain(&pending.domain, None) {
                    Ok(_) => whitelist_add_redirect(&pending.domain, pending.return_url.as_deref()),
                    Err(_) => "servo:whitelist?error=failed".to_owned(),
                };
                if let Ok(parsed) = Url::parse(&redirect) {
                    if parsed.scheme() == "servo" || is_navigation_allowed(&parsed) {
                        return redirect_response(request, &redirect);
                    }
                }
                redirect_response(request, "servo:whitelist")
            },

            #[cfg(feature = "kotisatama")]
            "whitelist/remove" => {
                use kotisatama_whitelist::normalize_domain;

                let domain = query_param(url.as_url(), "domain");
                let domain = match domain.and_then(|domain| normalize_domain(&domain).ok()) {
                    Some(domain) => domain,
                    None => return redirect_response(request, "servo:whitelist?error=invalid"),
                };
                let token = register_pending_whitelist_change("remove", domain.clone(), None);
                whitelist_confirm_response(request, "remove", &domain, &token)
            },

            #[cfg(feature = "kotisatama")]
            "whitelist/commit-remove" => {
                use kotisatama_whitelist::remove_user_domain;

                let domain = query_param(url.as_url(), "domain");
                let token = query_param(url.as_url(), "token");
                let pending = match (domain, token) {
                    (Some(domain), Some(token)) => {
                        take_pending_whitelist_change("remove", &domain, &token)
                    },
                    _ => None,
                };
                let Some(pending) = pending else {
                    return redirect_response(request, "servo:whitelist?error=failed");
                };
                match remove_user_domain(&pending.domain) {
                    Ok(_) => redirect_response(request, "servo:whitelist"),
                    Err(_) => redirect_response(request, "servo:whitelist?error=failed"),
                }
            },

            #[cfg(feature = "kotisatama")]
            "whitelist/list" => {
                let entries = kotisatama_whitelist::user_entries();
                let body = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_owned());
                json_response(request, body)
            },

            #[cfg(feature = "kotisatama")]
            "whitelist" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/my-sites.html",
            ),

            // KOTISATAMA-PATCH: tallenna kielivalinta desktop-UI:lle ja ohjaa takaisin configiin.
            #[cfg(feature = "kotisatama")]
            "locale" => {
                if let Some(set) = url
                    .as_url()
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

#[cfg(feature = "kotisatama")]
fn query_param(url: &url::Url, key: &str) -> Option<String> {
    url.query_pairs()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.into_owned())
        .filter(|value| !value.trim().is_empty())
}

#[cfg(feature = "kotisatama")]
#[derive(Debug, Clone)]
struct PendingWhitelistChange {
    action: &'static str,
    domain: String,
    return_url: Option<String>,
}

#[cfg(feature = "kotisatama")]
static WHITELIST_TOKENS: OnceLock<Mutex<HashMap<String, PendingWhitelistChange>>> =
    OnceLock::new();

#[cfg(feature = "kotisatama")]
static WHITELIST_TOKEN_COUNTER: AtomicU64 = AtomicU64::new(1);

#[cfg(feature = "kotisatama")]
fn whitelist_tokens() -> &'static Mutex<HashMap<String, PendingWhitelistChange>> {
    WHITELIST_TOKENS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(feature = "kotisatama")]
fn register_pending_whitelist_change(
    action: &'static str,
    domain: String,
    return_url: Option<String>,
) -> String {
    let count = WHITELIST_TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let token = format!("{nanos:x}-{count:x}");
    if let Ok(mut tokens) = whitelist_tokens().lock() {
        tokens.insert(
            token.clone(),
            PendingWhitelistChange {
                action,
                domain,
                return_url,
            },
        );
    }
    token
}

#[cfg(feature = "kotisatama")]
fn take_pending_whitelist_change(
    action: &'static str,
    domain: &str,
    token: &str,
) -> Option<PendingWhitelistChange> {
    let mut tokens = whitelist_tokens().lock().ok()?;
    let pending = tokens.remove(token)?;
    if pending.action == action && pending.domain.eq_ignore_ascii_case(domain) {
        Some(pending)
    } else {
        None
    }
}

#[cfg(feature = "kotisatama")]
fn whitelist_confirm_response(
    request: &Request,
    action: &'static str,
    domain: &str,
    token: &str,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    let verb = if action == "remove" {
        "Poistetaanko"
    } else {
        "Lisataanko"
    };
    let description = if action == "remove" {
        "Tama poistaa sivun vain omista sivuistasi. Kuratoituja valkoisia sivuja ei voi poistaa."
    } else {
        "Tama sallii sivun vain talla laitteella. Se ei muuta kuratoitua valkoisten sivujen listaa."
    };
    let path = if action == "remove" {
        "whitelist/commit-remove"
    } else {
        "whitelist/commit-add"
    };
    let confirm_href = format!(
        "servo:{path}?domain={}&token={}",
        encode_query_value(domain),
        encode_query_value(token)
    );
    let title = format!("{verb} {domain} kotisatamaan?");
    html_response(
        request,
        format!(
            "<!DOCTYPE html><html><head><meta charset=\"utf-8\">\
             <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\
             <title>{}</title>\
             <style>body{{font-family:system-ui,sans-serif;margin:2.5rem;line-height:1.5;max-width:42rem}}\
             a.button{{display:inline-block;margin-right:.75rem;padding:.55rem 1rem;border-radius:8px;\
             background:#1f6b4a;color:#fff;text-decoration:none}}a.cancel{{color:#555}}</style>\
             </head><body><main><h1>{}</h1><p>{}</p>\
             <p><a class=\"button\" href=\"{}\">Vahvista</a>\
             <a class=\"cancel\" href=\"servo:whitelist\">Peruuta</a></p></main></body></html>",
            escape_html(&title),
            escape_html(&title),
            escape_html(description),
            escape_html(&confirm_href)
        ),
    )
}

#[cfg(feature = "kotisatama")]
fn whitelist_add_redirect(domain: &str, return_url: Option<&str>) -> String {
    if let Some(return_url) = return_url {
        if return_url == "servo:whitelist" {
            return return_url.to_owned();
        }
        if let Ok(parsed) = url::Url::parse(return_url) {
            if parsed.scheme() == "http" || parsed.scheme() == "https" {
                return return_url.to_owned();
            }
        }
    }
    format!("https://{domain}/")
}

#[cfg(feature = "kotisatama")]
fn encode_query_value(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

#[cfg(feature = "kotisatama")]
fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(feature = "kotisatama")]
fn redirect_response(
    request: &Request,
    target: &str,
) -> Pin<Box<dyn Future<Output = Response> + Send>> {
    let json_target =
        serde_json::to_string(target).unwrap_or_else(|_| "\"servo:whitelist\"".to_owned());
    html_response(
        request,
        format!(
            "<!DOCTYPE html><html><head><meta charset=\"utf-8\">\
             <script>location.replace({json_target});</script></head><body></body></html>"
        ),
    )
}
