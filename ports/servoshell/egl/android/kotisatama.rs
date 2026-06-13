/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! JNI entry points for Kotisatama search and reports on Android.

use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jstring};
use jni::{Env, EnvUnowned};
use jni::errors::ThrowRuntimeExAndDefault;
use kotisatama_report::ReportKind;
use serde_json::json;

use crate::kotisatama::{self, SearchOutcome};

fn jstring_from(env: &mut Env, value: String) -> jstring {
    env.new_string(value)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_kotisatamaSearch<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    query: JString<'local>,
) -> jstring {
    env.with_env(|env| -> jni::errors::Result<_> {
        let query = query.to_string();
        let panel = kotisatama::search(&query);
        let body = match panel.outcome {
            SearchOutcome::Hits(hits) => {
                let hits = hits.iter().map(|hit| {
                    json!({
                        "id": hit.id,
                        "url": hit.url,
                        "title": hit.title,
                    })
                });
                json!({
                    "type": "hits",
                    "query": panel.query,
                    "hits": hits.collect::<Vec<_>>(),
                })
            },
            SearchOutcome::NoResults => json!({
                "type": "no_results",
                "query": panel.query,
            }),
            SearchOutcome::Error(message) => json!({
                "type": "error",
                "query": panel.query,
                "message": message,
            }),
        };
        Ok(jstring_from(env, body.to_string()))
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_kotisatamaSubmitReport<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    kind: JString<'local>,
    domain: JString<'local>,
    message: JString<'local>,
    context_url: JString<'local>,
) -> jstring {
    env.with_env(|env| -> jni::errors::Result<_> {
        let kind = parse_report_kind(&kind.to_string());
        let domain = domain.to_string();
        let message = message.to_string();
        let context_url = context_url.to_string();
        let context_url = if context_url.is_empty() {
            None
        } else {
            Some(context_url)
        };

        let form = kotisatama::KotisatamaReportForm {
            kind,
            domain,
            message,
        };

        let result = kotisatama::submit_report(&form, context_url)
            .map(|()| String::new())
            .unwrap_or_else(|error| error.to_string());
        Ok(jstring_from(env, result))
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_kotisatamaShouldShowReport<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    current_url: JString<'local>,
) -> jboolean {
    env.with_env(|env| -> jni::errors::Result<_> {
        let show = kotisatama::should_show_report_button(&current_url.to_string());
        Ok(show as jboolean)
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

fn parse_report_kind(raw: &str) -> ReportKind {
    match raw {
        "suggest_site" => ReportKind::SuggestSite,
        _ => ReportKind::SiteBroken,
    }
}
