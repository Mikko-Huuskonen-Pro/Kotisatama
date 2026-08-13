/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// KOTISATAMA-PATCH: android-moduuli julkinen clipboard_delegate-kytkentään — 公开android模块以便挂接剪贴板委托。
#[cfg(target_os = "android")]
pub mod android;
pub(crate) mod app;
mod host_trait;
mod log;
#[cfg(target_env = "ohos")]
mod ohos;
