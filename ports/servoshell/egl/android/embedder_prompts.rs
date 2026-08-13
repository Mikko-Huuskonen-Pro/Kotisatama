/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// KOTISATAMA-PATCH: <select>- ja context menu -promptit elossa JNI→Kotlin-UI:lle — 保持<select>与右键菜单提示存活并经JNI交给Kotlin UI。

use std::cell::RefCell;

use embedder_traits::embedder_controls::{ContextMenuAction, ContextMenuItem, SelectElementOptionOrOptgroup};
use jni::errors::Error;
use jni::{Env, jni_sig, jni_str};
use log::warn;
use serde_json::json;
use servo::{ContextMenu, SelectElement};

use super::callback_ref;

thread_local! {
    static PENDING_SELECT: RefCell<Option<SelectElement>> = const { RefCell::new(None) };
    static PENDING_CONTEXT_MENU: RefCell<Option<ContextMenu>> = const { RefCell::new(None) };
}

pub fn show_select_element(prompt: SelectElement) {
    PENDING_SELECT.with(|pending| {
        pending.replace(None);
        pending.replace(Some(prompt));
    });
    if let Err(error) = with_env(|env| {
        let json = select_element_json();
        let jstring = env.new_string(json)?;
        env.call_method(
            callback_ref(),
            jni_str!("onShowSelectElement"),
            jni_sig!("(Ljava/lang/String;)V"),
            &[(&jstring).into()],
        )?;
        Ok(())
    }) {
        warn!("onShowSelectElement JNI failed: {error:?}");
        dismiss_select_element();
    }
}

pub fn show_context_menu(menu: ContextMenu) {
    PENDING_CONTEXT_MENU.with(|pending| {
        pending.replace(None);
        pending.replace(Some(menu));
    });
    if let Err(error) = with_env(|env| {
        let json = context_menu_json();
        let jstring = env.new_string(json)?;
        env.call_method(
            callback_ref(),
            jni_str!("onShowContextMenu"),
            jni_sig!("(Ljava/lang/String;)V"),
            &[(&jstring).into()],
        )?;
        Ok(())
    }) {
        warn!("onShowContextMenu JNI failed: {error:?}");
        dismiss_context_menu();
    }
}

pub fn submit_select_element(selected_ids: Vec<usize>) {
    PENDING_SELECT.with(|pending| {
        if let Some(mut prompt) = pending.borrow_mut().take() {
            prompt.select(selected_ids);
            prompt.submit();
        }
    });
}

pub fn dismiss_select_element() {
    PENDING_SELECT.with(|pending| {
        pending.borrow_mut().take();
    });
}

pub fn submit_context_menu_action(action: i32) {
    PENDING_CONTEXT_MENU.with(|pending| {
        if let Some(menu) = pending.borrow_mut().take() {
            if let Some(action) = context_menu_action_from_i32(action) {
                menu.select(action);
            } else {
                menu.dismiss();
            }
        }
    });
}

pub fn dismiss_context_menu() {
    PENDING_CONTEXT_MENU.with(|pending| {
        if let Some(menu) = pending.borrow_mut().take() {
            menu.dismiss();
        }
    });
}

fn select_element_json() -> String {
    PENDING_SELECT.with(|pending| {
        let guard = pending.borrow();
        let Some(prompt) = guard.as_ref() else {
            return "{}".to_owned();
        };
        let options: Vec<_> = prompt.options().iter().map(|entry| match entry {
            SelectElementOptionOrOptgroup::Option(option) => json!({
                "type": "option",
                "id": option.id,
                "label": option.label,
                "disabled": option.is_disabled,
            }),
            SelectElementOptionOrOptgroup::Optgroup { label, options } => json!({
                "type": "optgroup",
                "label": label,
                "options": options.iter().map(|option| json!({
                    "id": option.id,
                    "label": option.label,
                    "disabled": option.is_disabled,
                })).collect::<Vec<_>>(),
            }),
        }).collect();
        json!({
            "allowMultiple": prompt.allow_select_multiple(),
            "selected": prompt.selected_options(),
            "options": options,
        })
        .to_string()
    })
}

fn context_menu_json() -> String {
    PENDING_CONTEXT_MENU.with(|pending| {
        let guard = pending.borrow();
        let Some(menu) = guard.as_ref() else {
            return "{}".to_owned();
        };
        let items: Vec<_> = menu.items().iter().map(|item| match item {
            ContextMenuItem::Item {
                label,
                action,
                enabled,
            } => json!({
                "type": "item",
                "label": label,
                "action": context_menu_action_to_i32(*action),
                "enabled": enabled,
            }),
            ContextMenuItem::Separator => json!({ "type": "separator" }),
        }).collect();
        json!({ "items": items }).to_string()
    })
}

fn context_menu_action_to_i32(action: ContextMenuAction) -> i32 {
    match action {
        ContextMenuAction::GoBack => 0,
        ContextMenuAction::GoForward => 1,
        ContextMenuAction::Reload => 2,
        ContextMenuAction::CopyLink => 3,
        ContextMenuAction::OpenLinkInNewWebView => 4,
        ContextMenuAction::CopyImageLink => 5,
        ContextMenuAction::OpenImageInNewView => 6,
        ContextMenuAction::Cut => 7,
        ContextMenuAction::Copy => 8,
        ContextMenuAction::Paste => 9,
        ContextMenuAction::SelectAll => 10,
    }
}

fn context_menu_action_from_i32(value: i32) -> Option<ContextMenuAction> {
    match value {
        0 => Some(ContextMenuAction::GoBack),
        1 => Some(ContextMenuAction::GoForward),
        2 => Some(ContextMenuAction::Reload),
        3 => Some(ContextMenuAction::CopyLink),
        4 => Some(ContextMenuAction::OpenLinkInNewWebView),
        5 => Some(ContextMenuAction::CopyImageLink),
        6 => Some(ContextMenuAction::OpenImageInNewView),
        7 => Some(ContextMenuAction::Cut),
        8 => Some(ContextMenuAction::Copy),
        9 => Some(ContextMenuAction::Paste),
        10 => Some(ContextMenuAction::SelectAll),
        _ => None,
    }
}

fn with_env(
    callback: impl FnOnce(&mut Env) -> Result<(), Error>,
) -> Result<(), Error> {
    super::with_attached_env(callback)
}
