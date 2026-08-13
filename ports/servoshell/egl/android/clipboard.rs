/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// KOTISATAMA-PATCH: Android ClipboardManager JNI-sillan kautta (ei arboard) — 通过JNI桥接Android剪贴板（不用arboard）。

use jni::objects::JString;
use jni::{jni_sig, jni_str};
use log::warn;
use servo::{ClipboardDelegate, StringRequest, WebView};

use super::callback_ref;

pub struct AndroidClipboardDelegate;

impl ClipboardDelegate for AndroidClipboardDelegate {
    fn clear(&self, _webview: WebView) {
        if let Err(error) = super::with_attached_env(|env| {
            env.call_method(
                callback_ref(),
                jni_str!("clipboardClear"),
                jni_sig!("()V"),
                &[],
            )?;
            Ok(())
        }) {
            warn!("Android clipboard clear failed: {error:?}");
        }
    }

    fn get_text(&self, _webview: WebView, request: StringRequest) {
        match super::with_attached_env(|env| {
            let result = env.call_method(
                callback_ref(),
                jni_str!("clipboardGetText"),
                jni_sig!("()Ljava/lang/String;"),
                &[],
            )?;
            let obj = result.l()?;
            JString::cast_local(env, obj)?.try_to_string(env)
        }) {
            Ok(text) => request.success(text),
            Err(error) => {
                warn!("Android clipboard get_text failed: {error:?}");
                request.failure(error.to_string());
            },
        }
    }

    fn set_text(&self, _webview: WebView, new_contents: String) {
        if let Err(error) = super::with_attached_env(|env| {
            let jstring = env.new_string(&new_contents)?;
            env.call_method(
                callback_ref(),
                jni_str!("clipboardSetText"),
                jni_sig!("(Ljava/lang/String;)V"),
                &[(&jstring).into()],
            )?;
            Ok(())
        }) {
            warn!("Android clipboard set_text failed: {error:?}");
        }
    }
}
