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
//! - servo:haku
//! - servo:pulloposti
//! - servo:varustamo
//! - servo:missa-olen
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
use std::sync::{Mutex, OnceLock};

use headers::{ContentType, HeaderMapExt};
use servo::UserAgentPlatform;
use servo::protocol_handler::{
    DoneChannel, FetchContext, NetworkError, ProtocolHandler, Request, ResourceFetchTiming,
    Response, ResponseBody,
};

#[cfg(feature = "kotisatama")]
use kotisatama_i18n::parse_locale_choice;

use crate::prefs::EXPERIMENTAL_PREFS;
use crate::protocols::resource::ResourceProtocolHandler;

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
            // KOTISATAMA-PATCH: sisäiset avomeri- ja pulloposti-sivut (resource_protocol) — 内部avomeri和漂流瓶页面（resource_protocol）。
            "avomeri" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/avomeri.html",
            ),

            #[cfg(feature = "kotisatama")]
            "avomeri/open" => {
                if !crate::kotisatama::product_profile().can_enter_avomeri() {
                    return redirect_response(request, "servo:avomeri?error=disabled");
                }
                let query = query_param(url.as_url(), "q").unwrap_or_default();
                // KOTISATAMA-PATCH: pending enter — seuraava top-level load syö lipun ja asettaa Avomerin — 待进入——下一个顶层加载消费标志并设置Avomeri。
                crate::kotisatama::enter_avomeri_mode_for_active_webview();
                return redirect_response(
                    request,
                    crate::kotisatama::avomeri_open_url(&query).as_str(),
                );
            },

            #[cfg(feature = "kotisatama")]
            "avomeri/leave" => {
                // KOTISATAMA-PATCH: pending leave — seuraava load syö lipun — 待离开——下一个加载消费标志。
                crate::kotisatama::leave_avomeri_mode_for_active_webview();
                return redirect_response(request, "servo:newtab");
            },

            // KOTISATAMA-PATCH: sisäinen hakutulossivu (resource_protocol/haku.html) — 内部搜索结果页面（resource_protocol/haku.html）。
            #[cfg(feature = "kotisatama")]
            "haku" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/haku.html",
            ),

            #[cfg(feature = "kotisatama")]
            "haku/data" => {
                let query = query_param(url.as_url(), "q").unwrap_or_default();
                let body = crate::kotisatama::search_results_json(&query);
                return json_response(request, body);
            },

            // KOTISATAMA-PATCH: Wikipedia-offline-snapshot (slug → paikallinen HTML) — Wikipedia离线快照。
            #[cfg(feature = "kotisatama")]
            "wiki" => {
                let slug = query_param(url.as_url(), "slug").unwrap_or_default();
                let body = crate::kotisatama::wiki_snapshot_html(&slug);
                return html_response(request, body);
            },

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

            #[cfg(feature = "kotisatama")]
            "varustamo/registry" => {
                if !crate::kotisatama::varustamo_enabled() {
                    return json_response(request, r#"{"apps":[],"parked":true}"#.to_string());
                }
                let body = kotisatama_varustamo::load_registry_json()
                    .unwrap_or_else(|error| format!(r#"{{"error":"{error}"}}"#));
                json_response(request, body)
            },

            #[cfg(feature = "kotisatama")]
            "varustamo/app" => {
                if !crate::kotisatama::varustamo_enabled() {
                    return redirect_response(request, "servo:newtab");
                }
                let app_id = query_param(url.as_url(), "id").unwrap_or_default();
                let target = match app_id.as_str() {
                    "pulloposti" => {
                        std::thread::spawn(crate::kotisatama::ensure_pulloposti);
                        "servo:pulloposti"
                    },
                    "missa-olen" => {
                        std::thread::spawn(crate::kotisatama::ensure_missa_olen);
                        "servo:missa-olen"
                    },
                    _ => "servo:varustamo",
                };
                return redirect_response(request, target);
            },

            #[cfg(feature = "kotisatama")]
            "varustamo" => {
                if !crate::kotisatama::varustamo_enabled() {
                    return redirect_response(request, "servo:newtab");
                }
                ResourceProtocolHandler::response_for_path(
                    request,
                    done_chan,
                    context,
                    "/varustamo.html",
                )
            },

            "missa-olen" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/missa-olen.html",
            ),

            // KOTISATAMA-PATCH: whitelist-blokkaussivu (i18n: blocked.html + kotisatama-i18n.js) — 白名单阻止页面（国际化：blocked.html + kotisatama-i18n.js）。
            "blocked" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/blocked.html",
            ),

            // KOTISATAMA-PATCH: käyttäjän omat whitelist-sivut (overlay, ei CDN:ää) — 用户自定义白名单页面（覆盖，无CDN）。
            #[cfg(feature = "kotisatama")]
            "whitelist/add" => {
                use kotisatama_whitelist::normalize_domain;

                if !crate::kotisatama::product_profile().can_add_user_domain() {
                    return redirect_response(request, "servo:whitelist?error=disabled");
                }

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

                if !crate::kotisatama::product_profile().can_add_user_domain() {
                    return redirect_response(request, "servo:whitelist?error=disabled");
                }

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
                    Ok(_) => {
                        // KOTISATAMA-PATCH: päivitä hakuindeksi heti lisäyksen jälkeen — 添加后立即刷新搜索索引。
                        crate::kotisatama::reload_search_index();
                        whitelist_add_redirect(&pending.domain, pending.return_url.as_deref())
                    },
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
                    Ok(_) => {
                        // KOTISATAMA-PATCH: päivitä hakuindeksi poiston jälkeen — 删除后刷新搜索索引。
                        crate::kotisatama::reload_search_index();
                        redirect_response(request, "servo:whitelist")
                    },
                    Err(_) => redirect_response(request, "servo:whitelist?error=failed"),
                }
            },

            #[cfg(feature = "kotisatama")]
            "whitelist/list" => {
                let entries = kotisatama_whitelist::user_entries();
                let body = serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_owned());
                json_response(request, body)
            },

            // KOTISATAMA-PATCH: profiilitila JSON-muodossa config.html:lle — 向config.html提供配置文件JSON状态。
            #[cfg(feature = "kotisatama")]
            "profile/data" => {
                use kotisatama_whitelist::{current_profile_state, profile_restrictions_active};
                let state = current_profile_state();
                let body = serde_json::json!({
                    "profile": state.profile.as_str(),
                    "avomeri_enabled": state.avomeri_enabled,
                    "first_run_completed": state.first_run_completed,
                    "has_emoji": state.emoji_hash.is_some(),
                    "restrictions_active": profile_restrictions_active(state.profile),
                    "locked_out": state.is_locked_out(),
                    "lockout_remaining_secs": state.lockout_remaining_secs(),
                    "failed_attempts": state.failed_attempts,
                });
                json_response(request, body.to_string())
            },

            // KOTISATAMA-PATCH: tarkista emoji-salasana ilman profiilinvaihtoa — 验证表情密码（不切换配置文件）。
            #[cfg(feature = "kotisatama")]
            "profile/verify" => {
                use kotisatama_whitelist::{EmojiAuthResult, verify_emoji_password};

                let emoji_param = query_param(url.as_url(), "emoji");
                let emojis: Option<Vec<char>> = emoji_param.map(|s| s.chars().collect());
                let auth = match emojis.as_deref() {
                    Some(e) => verify_emoji_password(e).unwrap_or(EmojiAuthResult::Wrong),
                    None => EmojiAuthResult::Wrong,
                };
                let (status, message) = match auth {
                    EmojiAuthResult::Ok => ("ok", "Salasana oikein."),
                    EmojiAuthResult::NotRequired => ("ok", "Lukkoa ei tarvita."),
                    EmojiAuthResult::Wrong => ("wrong", "Emoji-salasana väärin tai puuttuu."),
                    EmojiAuthResult::LockedOut => {
                        ("locked", "Liian monta yritystä. Odota 5 minuuttia.")
                    },
                };
                let body = serde_json::json!({ "status": status, "message": message });
                json_response(request, body.to_string())
            },

            // KOTISATAMA-PATCH: aseta profiili / avomeri / emoji (query: profile, avomeri, emoji) — 设置配置文件/Avomeri/表情密码。
            #[cfg(feature = "kotisatama")]
            "profile/set" => {
                use kotisatama_whitelist::{
                    EmojiAuthResult, Profile, set_avomeri_enabled, set_profile,
                };

                let profile_param = query_param(url.as_url(), "profile");
                let avomeri_param = query_param(url.as_url(), "avomeri");
                let emoji_param = query_param(url.as_url(), "emoji");
                let emojis: Option<Vec<char>> = emoji_param.map(|s| s.chars().collect());
                let emoji_slice = emojis.as_deref();
                let profile_changed = profile_param.is_some();

                let result = (|| {
                    if let Some(name) = profile_param {
                        let profile = Profile::parse(&name).ok_or(EmojiAuthResult::Wrong)?;
                        let profile_result =
                            set_profile(profile, emoji_slice).map_err(|_| EmojiAuthResult::Wrong)?;
                        if !matches!(
                            profile_result,
                            EmojiAuthResult::Ok | EmojiAuthResult::NotRequired
                        ) {
                            return Ok(profile_result);
                        }
                        if let Some(avomeri) = avomeri_param {
                            let enabled = matches!(
                                avomeri.trim().to_ascii_lowercase().as_str(),
                                "1" | "true" | "yes" | "on"
                            );
                            return set_avomeri_enabled(enabled, emoji_slice)
                                .map_err(|_| EmojiAuthResult::Wrong);
                        }
                        return Ok(profile_result);
                    }
                    if let Some(avomeri) = avomeri_param {
                        let enabled = matches!(
                            avomeri.trim().to_ascii_lowercase().as_str(),
                            "1" | "true" | "yes" | "on"
                        );
                        return set_avomeri_enabled(enabled, emoji_slice)
                            .map_err(|_| EmojiAuthResult::Wrong);
                    }
                    Ok(EmojiAuthResult::Wrong)
                })();

                let auth = result.unwrap_or(EmojiAuthResult::Wrong);
                let (status, message) = match auth {
                    EmojiAuthResult::Ok => ("ok", "Profiili päivitetty."),
                    EmojiAuthResult::NotRequired => ("ok", "Tallennettu."),
                    EmojiAuthResult::Wrong => ("wrong", "Emoji-salasana väärin tai puuttuu."),
                    EmojiAuthResult::LockedOut => {
                        ("locked", "Liian monta yritystä. Odota 5 minuuttia.")
                    },
                };
                // KOTISATAMA-PATCH: whitelist hot-reload profiilinvaihdon jälkeen — 切换配置文件后热重载白名单。
                if status == "ok" && profile_changed {
                    let profile = kotisatama_whitelist::effective_whitelist_profile();
                    let cache = kotisatama_search::cached_whitelist_path();
                    if let Err(error) =
                        kotisatama_whitelist::reload_for_profile(cache, profile)
                    {
                        log::warn!("Kotisatama: whitelist reload after profile switch failed: {error}");
                    }
                    // KOTISATAMA-PATCH: profiilivaihto pakottaa Avomeri-poiston — 配置文件切换强制退出Avomeri。
                    crate::kotisatama::leave_avomeri_mode_all();
                }
                let body = serde_json::json!({ "status": status, "message": message });
                json_response(request, body.to_string())
            },

            #[cfg(feature = "kotisatama")]
            "whitelist" => ResourceProtocolHandler::response_for_path(
                request,
                done_chan,
                context,
                "/my-sites.html",
            ),

            // KOTISATAMA-PATCH: tallenna kielivalinta desktop-UI:lle ja ohjaa takaisin configiin — 保存桌面UI的语言选择并重定向回配置。
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
    created_at: std::time::Instant,
}

#[cfg(feature = "kotisatama")]
static WHITELIST_TOKENS: OnceLock<Mutex<HashMap<String, PendingWhitelistChange>>> = OnceLock::new();

#[cfg(feature = "kotisatama")]
const TOKEN_TTL: std::time::Duration = std::time::Duration::from_secs(10 * 60);
#[cfg(feature = "kotisatama")]
const MAX_PENDING: usize = 64;

#[cfg(feature = "kotisatama")]
fn whitelist_tokens() -> &'static Mutex<HashMap<String, PendingWhitelistChange>> {
    WHITELIST_TOKENS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(feature = "kotisatama")]
fn new_whitelist_token() -> String {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};
    // 128 bittiä satunnaisuutta (RandomState + 2 hasheria)
    let s = RandomState::new();
    let mut h1 = s.build_hasher();
    h1.write(b"kotisatama-whitelist-token");
    let a = h1.finish();
    let mut h2 = s.build_hasher();
    h2.write_u64(a);
    let b = h2.finish();
    format!("{a:016x}{b:016x}")
}

#[cfg(feature = "kotisatama")]
fn prune_whitelist_tokens(tokens: &mut HashMap<String, PendingWhitelistChange>) {
    let now = std::time::Instant::now();
    tokens.retain(|_, pending| now.duration_since(pending.created_at) <= TOKEN_TTL);
    if tokens.len() > MAX_PENDING {
        let mut entries: Vec<_> = tokens
            .iter()
            .map(|(k, v)| (k.clone(), v.created_at))
            .collect();
        entries.sort_by_key(|(_, created_at)| *created_at);
        for (token, _) in entries.into_iter().take(tokens.len() - MAX_PENDING) {
            tokens.remove(&token);
        }
    }
}

#[cfg(feature = "kotisatama")]
fn register_pending_whitelist_change(
    action: &'static str,
    domain: String,
    return_url: Option<String>,
) -> String {
    let token = new_whitelist_token();
    if let Ok(mut tokens) = whitelist_tokens().lock() {
        prune_whitelist_tokens(&mut tokens);
        tokens.insert(
            token.clone(),
            PendingWhitelistChange {
                action,
                domain,
                return_url,
                created_at: std::time::Instant::now(),
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
    prune_whitelist_tokens(&mut tokens);
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
    let title = format!("{verb} {domain} satamaan?");
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
