/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![allow(non_snake_case)]

// KOTISATAMA-PATCH: JNI-sillat Android-hakuun ja raportointiin — Android搜索和报告的JNI桥接。
#[cfg(feature = "kotisatama")]
mod kotisatama;
// KOTISATAMA-PATCH: leikepöytä + select/context-menu -promptit — 剪贴板与select/右键菜单提示。
pub mod clipboard;
mod embedder_prompts;

use std::cell::RefCell;
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};

use android_logger::{self, Config, FilterBuilder};
use euclid::{Point2D, Rect, Scale, Size2D};
use jni::errors::{Error, ThrowRuntimeExAndDefault};
use jni::objects::{Global, JClass, JIntArray, JObject, JString, JValue, JValueOwned};
use jni::strings::JNIStr;
use jni::sys::{jboolean, jfloat, jint, jobject};
use jni::{Env, EnvUnowned, JavaVM, jni_sig, jni_str};
use keyboard_types::{Key, NamedKey};
use log::{debug, error, info, warn};
use raw_window_handle::{
    AndroidDisplayHandle, AndroidNdkWindowHandle, DisplayHandle, RawDisplayHandle, RawWindowHandle,
    WindowHandle,
};
pub use servo::MediaSessionPlaybackState;
use servo::{
    self, ContextMenu, DevicePixel, EventLoopWaker, InputMethodControl, LoadStatus,
    MediaSessionActionType, MouseButton, PrefValue, SelectElement, WebViewId,
};

use super::app::{App, AppInitOptions};
use super::host_trait::HostTrait;
use crate::prefs::{ArgumentParsingResult, EXPERIMENTAL_PREFS, parse_command_line_arguments};

thread_local! {
    pub static APP: RefCell<Option<Rc<App>>> = const { RefCell::new(None) };
}

static CALLBACK_OBJECT: OnceLock<Global<JObject<'static>>> = OnceLock::new();
// KOTISATAMA-PATCH: jaettu JVM clipboard/prompt-JNI-kutsuille — 供剪贴板/提示JNI调用的共享JVM。
static ANDROID_JVM: OnceLock<JavaVM> = OnceLock::new();

fn callback_ref() -> &'static JObject<'static> {
    CALLBACK_OBJECT.get().expect("Servo init failed").as_ref()
}

struct HostCallbacks {
    jvm: JavaVM,
}

unsafe extern "C" {
    fn ANativeWindow_fromSurface(env: *mut jni::sys::JNIEnv, surface: jobject) -> *mut c_void;
}

#[unsafe(no_mangle)]
pub extern "C" fn android_main() {
    // FIXME(mukilan): this android_main is only present to stop
    // the java side 'System.loadLibrary('servoshell') call from
    // failing due to undefined reference to android_main introduced
    // by winit's android-activity crate. There is no way to disable
    // this currently.
}

fn call<F>(env: &mut Env, f: F)
where
    F: FnOnce(&App),
{
    APP.with(|app| match app.borrow().as_ref() {
        Some(app) => (f)(app),
        None => throw(env, jni_str!("Servo not available in this thread")),
    });
}

// KOTISATAMA-PATCH: liitä nykyinen säie JVM:ään clipboard/prompt-kutsuille — 将当前线程附加到JVM供剪贴板/提示调用。
pub(crate) fn with_attached_env<ResultType>(
    callback: impl FnOnce(&mut Env) -> Result<ResultType, Error>,
) -> Result<ResultType, Error> {
    ANDROID_JVM
        .get()
        .expect("Android JVM not initialized")
        .attach_current_thread(callback)
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_version<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> JString<'local> {
    let version = crate::VERSION;
    env.with_env(|env| -> jni::errors::Result<_> { env.new_string(version) })
        .resolve::<ThrowRuntimeExAndDefault>()
}

/// Initialize Servo. At that point, we need a valid GL context. In the future, this will
/// be done in multiple steps.
#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_init<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    context: JObject<'local>,
    args: JString<'local>,
    url: JString<'local>,
    size: JObject<'local>,
    density: jfloat,
    logStr: JString<'local>,
    log: jboolean,
    experimental_mode: jboolean,
    callbacks_obj: JObject<'local>,
    surface: JObject<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        // Upstream: InitOptions inlined (servo#46994). Kotisatama keeps map_err-käsittely alempana.
        // 上游：InitOptions 已内联；Kotisatama 在下方保留 map_err 错误处理。
        let args = JString::cast_local(env, args)?.try_to_string(env).ok();
        let url = JString::cast_local(env, url)?.try_to_string(env).ok();
        let log_str = JString::cast_local(env, logStr)?.try_to_string(env).ok();

        let viewport_rect = jni_coordinate_to_rust_viewport_rect(env, &size)?;

        let mut args: Vec<String> = args
            .and_then(|args| {
                serde_json::from_str(&args)
                    .inspect_err(|_| {
                        error!(
                            "Invalid arguments. Servo arguments must be formatted as a JSON array"
                        )
                    })
                    .ok()
            })
            .unwrap_or_default();

        if experimental_mode {
            args.push("--enable-experimental-web-platform-features".to_owned());
        }

        let (display_handle, window_handle) = display_and_window_handle(env, &surface);

        if log {
            // Note: Android debug logs are stripped from a release build.
            // debug!() will only show in a debug build. Use info!() if logs
            // should show up in adb logcat with a release build.
            let filters = [
                "servo",
                "servoshell",
                "servoshell::egl:gl_glue",
                // Show redirected stdout / stderr by default
                "servoshell::egl::log",
                // Show JS errors by default.
                "script::dom::bindings::error",
                // Show GL errors by default.
                "servo_canvas::webgl_thread",
                "paint::paint",
                "servo_constellation::constellation",
            ];
            let mut filter_builder = FilterBuilder::new();
            for &module in &filters {
                filter_builder.filter_module(module, log::LevelFilter::Debug);
            }
            if let Some(log_str) = log_str {
                for module in log_str.split(',') {
                    filter_builder.filter_module(module, log::LevelFilter::Debug);
                }
            }

            android_logger::init_once(
                Config::default()
                    .with_max_level(log::LevelFilter::Debug)
                    .with_filter(filter_builder.build())
                    .with_tag("servoshell"),
            );

            // In production mode we don't redirect stdout / stderr, so any
            // panic messages would be lost without this hook.
            std::panic::set_hook(Box::new(|info| {
                let current_thread = std::thread::current();
                let thread_name = current_thread.name().unwrap_or("<unnamed>");
                error!("Panic in Rust code (thread: {thread_name}):");
                error!("{info}");
            }));
        }

        info!("init");

        // We only redirect stdout and stderr for non-production builds, since it is
        // only used for debugging purposes. This saves us one thread in production.
        #[cfg(not(servo_production))]
        if let Err(e) = super::log::redirect_stdout_and_stderr() {
            error!("Failed to redirect stdout and stderr to logcat due to: {e:?}");
        }

        let callbacks: Global<JObject<'static>> = env.new_global_ref(callbacks_obj).map_err(|e| {
            error!("JNIServo_init: new_global_ref failed: {e:?}");
            e
        })?;

        if let Err(_already) = CALLBACK_OBJECT.set(callbacks) {
            error!("JNIServo_init: CALLBACK_OBJECT already set (re-init)");
            // Reuse the existing callback object instead of panicking on surface re-create.
        }

        let jvm = env.get_java_vm().map_err(|e| {
            error!("JNIServo_init: get_java_vm failed: {e:?}");
            e
        })?;
        // KOTISATAMA-PATCH: tallenna JVM clipboard/prompt-moduuleille — 保存JVM供剪贴板/提示模块使用。
        let _ = ANDROID_JVM.set(jvm.clone());
        let event_loop_waker = Box::new(WakeupCallback::new(jvm.clone()));

        let host = Rc::new(HostCallbacks::new(jvm));

        crate::init_crypto();

        if let Err(error) = set_default_config_dir(env, &context) {
            error!("Failed to determine Android config directory: {error:?}");
        }

        // KOTISATAMA-PATCH: profiilipolku + ensimmäinen käynnistys → servo:config
        // 设置配置文件路径；首次启动打开 servo:config。
        #[cfg(feature = "kotisatama")]
        let url = {
            if let Some(mut dir) = crate::prefs::default_config_dir() {
                dir.pop();
                dir.push("kotisatama");
                dir.push("profile.json");
                kotisatama_whitelist::set_profile_path(dir);
            }
            if url.is_none() {
                let state = kotisatama_whitelist::current_profile_state();
                if !state.first_run_completed {
                    Some("servo:config".to_owned())
                } else {
                    url
                }
            } else {
                url
            }
        };

        let (opts, mut preferences, servoshell_preferences) =
            match parse_command_line_arguments(args.as_slice()) {
                ArgumentParsingResult::ContentProcess(..) => {
                    unreachable!("Android does not have support for multiprocess yet.")
                },
                ArgumentParsingResult::ChromeProcess(opts, preferences, servoshell_preferences) => {
                    (opts, preferences, servoshell_preferences)
                },
                ArgumentParsingResult::Exit => {
                    std::process::exit(0);
                },
                ArgumentParsingResult::ErrorParsing => {
                    error!("JNIServo_init: argument parsing failed");
                    std::process::exit(1);
                },
            };

        preferences.set_value("viewport_meta_enabled", servo::PrefValue::Bool(true));

        crate::init_tracing(servoshell_preferences.tracing_filter.as_deref());

        let (display_handle, window_handle) = unsafe {
            (
                DisplayHandle::borrow_raw(display_handle),
                WindowHandle::borrow_raw(window_handle),
            )
        };

        let hidpi_scale_factor = Scale::new(density);

        APP.with(|app| {
            let new_app = App::new(AppInitOptions {
                host,
                event_loop_waker,
                initial_url: url,
                opts,
                preferences,
                servoshell_preferences,
                #[cfg(feature = "webxr")]
                xr_discovery: None,
            });
            new_app.add_platform_window(
                display_handle,
                window_handle,
                viewport_rect,
                hidpi_scale_factor,
                None,
            );
            *app.borrow_mut() = Some(new_app);
        });
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_setExperimentalMode<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    enable: jboolean,
) {
    debug!("setExperimentalMode {enable}");
    env.with_env(|env| -> jni::errors::Result<_> {
        call(env, |s| {
            for pref in EXPERIMENTAL_PREFS {
                s.servo().set_preference(pref, PrefValue::Bool(enable));
            }
        });
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_resize<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    size: JObject<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        let viewport_rect = jni_coordinate_to_rust_viewport_rect(env, &size)?;
        debug!("resize {viewport_rect:#?}");
        call(env, |s| s.resize(viewport_rect));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_performUpdates<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("performUpdates");
        call(env, |app| app.spin_event_loop());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_loadUri<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    url: JString<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("loadUri");
        call(env, |s| s.load_uri(&url.to_string()));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_reload<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("reload");
        call(env, |s| s.reload());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_stop<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("stop");
        call(env, |s| s.stop());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_goBack<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("goBack");
        call(env, |s| s.go_back());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_goForward<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("goForward");
        call(env, |s| s.go_forward());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

// KOTISATAMA-PATCH: Android-välilehdet (list/new/activate/close) — Android标签页API。
#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_listWebViews<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> jni::sys::jstring {
    env.with_env(|env| -> jni::errors::Result<_> {
        let json = APP.with(|app| match app.borrow().as_ref() {
            Some(app) => app.list_webviews_json(),
            None => "[]".to_owned(),
        });
        Ok(env
            .new_string(json)
            .map(|s| s.into_raw())
            .unwrap_or(std::ptr::null_mut()))
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_newWebView<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        APP.with(|app| match app.borrow().as_ref() {
            Some(app) => app.new_webview_blank(),
            None => throw(env, jni_str!("Servo not available in this thread")),
        });
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_activateWebView<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    index: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        let index = index.max(0) as usize;
        call(env, |s| s.activate_webview_index(index));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_closeWebView<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    index: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        let index = index.max(0) as usize;
        call(env, |s| s.close_webview_index(index));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_scroll<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    dx: jint,
    dy: jint,
    x: jint,
    y: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("scroll");
        call(env, |s| s.scroll(dx as f32, dy as f32, x as f32, y as f32));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_doFrame<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        call(env, |s| s.notify_vsync());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

enum KeyCode {
    Delete,
    ForwardDelete,
    Enter,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
}

impl TryFrom<i32> for KeyCode {
    type Error = ();

    // Values derived from <https://developer.android.com/reference/android/view/KeyEvent>
    fn try_from(keycode: i32) -> Result<KeyCode, ()> {
        Ok(match keycode {
            66 => KeyCode::Enter,
            67 => KeyCode::Delete,
            112 => KeyCode::ForwardDelete,
            21 => KeyCode::ArrowLeft,
            22 => KeyCode::ArrowRight,
            19 => KeyCode::ArrowUp,
            20 => KeyCode::ArrowDown,
            _ => return Err(()),
        })
    }
}

impl From<KeyCode> for Key {
    fn from(keycode: KeyCode) -> Key {
        Key::Named(match keycode {
            KeyCode::Enter => NamedKey::Enter,
            KeyCode::Delete => NamedKey::Backspace,
            KeyCode::ForwardDelete => NamedKey::Delete,
            KeyCode::ArrowLeft => NamedKey::ArrowLeft,
            KeyCode::ArrowRight => NamedKey::ArrowRight,
            KeyCode::ArrowUp => NamedKey::ArrowUp,
            KeyCode::ArrowDown => NamedKey::ArrowDown,
        })
    }
}

fn key_from_unicode_keycode(unicode: u32, keycode: i32) -> Option<Key> {
    // KOTISATAMA-PATCH: pehmonäppäimistön Enter lähettää keycode=66 + unicode '\n';
    // Servo tarvitsee NamedKey::Enter lomakkeen lähetykseen (text_input.rs handle_return).
    // 软键盘 Enter 同时发送 keycode=66 和 unicode '\n'；Servo 需要 NamedKey::Enter 才能提交表单。
    if let Ok(code) = KeyCode::try_from(keycode) {
        return Some(Key::from(code));
    }
    char::from_u32(unicode)
        .filter(|c| *c != '\0')
        .map(|c| Key::Character(String::from(c)))
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_keydown<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    keycode: jint,
    unicode: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("keydown {keycode}");
        if let Some(key) = key_from_unicode_keycode(unicode as u32, keycode) {
            call(env, move |s| s.key_down(key));
        }
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_keyup<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    keycode: jint,
    unicode: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("keyup {keycode}");
        if let Some(key) = key_from_unicode_keycode(unicode as u32, keycode) {
            call(env, move |s| s.key_up(key));
        }
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchDown<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("touchDown");
        call(env, |s| s.touch_down(x, y, pointer_id));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchUp<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("touchUp");
        call(env, |s| s.touch_up(x, y, pointer_id));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchMove<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("touchMove");
        call(env, |s| s.touch_move(x, y, pointer_id));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_touchCancel<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
    pointer_id: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("touchCancel");
        call(env, |s| s.touch_cancel(x, y, pointer_id));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoomStart<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jfloat,
    y: jfloat,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("pinchZoomStart");
        call(env, |s| s.pinchzoom_start(factor, x, y));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoom<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jfloat,
    y: jfloat,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("pinchZoom");
        call(env, |s| s.pinchzoom(factor, x, y));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pinchZoomEnd<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    factor: jfloat,
    x: jfloat,
    y: jfloat,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("pinchZoomEnd");
        call(env, |s| s.pinchzoom_end(factor, x, y));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_click(
    mut env: EnvUnowned,
    _: JClass,
    x: jfloat,
    y: jfloat,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("click");
        call(env, |s| {
            s.mouse_down(x, y, MouseButton::Left);
            s.mouse_up(x, y, MouseButton::Left);
        });
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_pausePainting(
    mut env: EnvUnowned,
    _: JClass<'_>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("pausePainting");
        call(env, |s| s.pause_painting());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_resumePainting<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    surface: JObject<'local>,
    size: JObject<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("resumePainting");
        let viewport_rect = jni_coordinate_to_rust_viewport_rect(env, &size)?;
        let (_, window_handle) = display_and_window_handle(env, &surface);

        call(env, |s| {
            s.resume_painting(window_handle, viewport_rect);
        });
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_mediaSessionAction<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    action: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        debug!("mediaSessionAction");

        let action = match action {
            1 => MediaSessionActionType::Play,
            2 => MediaSessionActionType::Pause,
            3 => MediaSessionActionType::SeekBackward,
            4 => MediaSessionActionType::SeekForward,
            5 => MediaSessionActionType::PreviousTrack,
            6 => MediaSessionActionType::NextTrack,
            7 => MediaSessionActionType::SkipAd,
            8 => MediaSessionActionType::Stop,
            9 => MediaSessionActionType::SeekTo,
            _ => {
                warn!("Ignoring unknown MediaSessionAction");
                return Ok(());
            },
        };
        call(env, |s| s.media_session_action(action.clone()));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

// KOTISATAMA-PATCH: Android IME → Servo (insert/delete/enter/dismiss) — Android输入法到Servo。
fn ime_delete_backward(app: &App, count: i32) {
    for _ in 0..count.max(0) {
        app.key_down(Key::Named(NamedKey::Backspace));
        app.key_up(Key::Named(NamedKey::Backspace));
    }
}

fn ime_delete_forward(app: &App, count: i32) {
    for _ in 0..count.max(0) {
        app.key_down(Key::Named(NamedKey::Delete));
        app.key_up(Key::Named(NamedKey::Delete));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_imeInsertText<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    text: JString<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        let text = JString::cast_local(env, text)?.try_to_string(env)?;
        call(env, |s| s.ime_insert_text(text));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_imeDeleteBackward<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    length: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        call(env, |s| ime_delete_backward(s, length));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_imeDeleteForward<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    length: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        call(env, |s| ime_delete_forward(s, length));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_imeSendEnter<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        call(env, |s| {
            s.key_down(Key::Named(NamedKey::Enter));
            s.key_up(Key::Named(NamedKey::Enter));
        });
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_imeDismissed<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        call(env, |s| s.ime_dismissed());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

// KOTISATAMA-PATCH: <select>-valinnan submit/dismiss JNI — <select>提交/取消的JNI。
#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_submitSelectElement<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    selected_ids: JIntArray<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        let length = selected_ids.len(env)?;
        let mut ids = vec![0i32; length];
        selected_ids.get_region(env, 0, &mut ids)?;
        embedder_prompts::submit_select_element(ids.into_iter().map(|id| id as usize).collect());
        call(env, |app| app.spin_event_loop());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_dismissSelectElement<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        embedder_prompts::dismiss_select_element();
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

// KOTISATAMA-PATCH: context menu + pitkä painallus (oikea klikkaus) — 右键菜单与长按触发。
#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_submitContextMenuAction<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    action: jint,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        embedder_prompts::submit_context_menu_action(action);
        call(env, |app| app.spin_event_loop());
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_dismissContextMenu<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        embedder_prompts::dismiss_context_menu();
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "C" fn Java_org_servo_servoview_JNIServo_showContextMenuAt<'local>(
    mut env: EnvUnowned<'local>,
    _: JClass<'local>,
    x: jfloat,
    y: jfloat,
) {
    env.with_env(|env| -> jni::errors::Result<_> {
        call(env, |app| app.show_context_menu_at(x, y));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

pub struct WakeupCallback {
    jvm: Arc<JavaVM>,
}

impl WakeupCallback {
    fn new(jvm: JavaVM) -> WakeupCallback {
        WakeupCallback { jvm: Arc::new(jvm) }
    }
}

impl EventLoopWaker for WakeupCallback {
    fn clone_box(&self) -> Box<dyn EventLoopWaker> {
        let jvm = self.jvm.clone();
        Box::new(WakeupCallback { jvm })
    }
    fn wake(&self) {
        debug!("wakeup");
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                env.call_method(callback_ref(), jni_str!("wakeup"), jni_sig!("()V"), &[])?;
                Ok(())
            })
            .unwrap();
    }
}

impl HostCallbacks {
    fn new(jvm: JavaVM) -> HostCallbacks {
        HostCallbacks { jvm }
    }
}

impl HostTrait for HostCallbacks {
    fn show_alert(&self, message: String) {
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let Ok(string) = new_string_as_jvalue(env, &message) else {
                    return Ok(());
                };
                env.call_method(
                    callback_ref(),
                    jni_str!("onAlert"),
                    jni_sig!("(Ljava/lang/String;)V"),
                    &[(&string).into()],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn notify_load_status_changed(&self, load_status: LoadStatus) {
        debug!("notify_load_status_changed: {load_status:?}");
        self.jvm
            .attach_current_thread(|env| match load_status {
                LoadStatus::Started => env
                    .call_method(
                        callback_ref(),
                        jni_str!("onLoadStarted"),
                        jni_sig!("()V"),
                        &[],
                    )
                    .map(|_| ()),
                LoadStatus::HeadParsed => Ok(()),
                LoadStatus::Complete => env
                    .call_method(
                        callback_ref(),
                        jni_str!("onLoadEnded"),
                        jni_sig!("()V"),
                        &[],
                    )
                    .map(|_| ()),
            })
            .unwrap();
    }

    fn on_shutdown_complete(&self) {
        debug!("on_shutdown_complete");
    }

    fn on_title_changed(&self, title: Option<String>) {
        debug!("on_title_changed");
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let title = title.unwrap_or_default();
                let Ok(title_string) = new_string_as_jvalue(env, &title) else {
                    return Ok(());
                };
                env.call_method(
                    callback_ref(),
                    jni_str!("onTitleChanged"),
                    jni_sig!("(Ljava/lang/String;)V"),
                    &[(&title_string).into()],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn on_url_changed(&self, url: String) {
        debug!("on_url_changed");
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let Ok(url_string) = new_string_as_jvalue(env, &url) else {
                    return Ok(());
                };

                env.call_method(
                    callback_ref(),
                    jni_str!("onUrlChanged"),
                    jni_sig!("(Ljava/lang/String;)V"),
                    &[(&url_string).into()],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn on_history_changed(&self, can_go_back: bool, can_go_forward: bool) {
        debug!("on_history_changed");
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let can_go_back = JValue::Bool(can_go_back as jboolean);
                let can_go_forward = JValue::Bool(can_go_forward as jboolean);
                env.call_method(
                    callback_ref(),
                    jni_str!("onHistoryChanged"),
                    jni_sig!("(ZZ)V"),
                    &[can_go_back, can_go_forward],
                )?;
                Ok(())
            })
            .unwrap();
    }

    // KOTISATAMA-PATCH: IME-show välittää multiline InputConnectionille — IME显示时把multiline传给InputConnection。
    fn on_ime_show(&self, control: InputMethodControl) {
        let multiline = control.multiline();
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let multiline = JValue::Bool(multiline as jboolean);
                env.call_method(
                    callback_ref(),
                    jni_str!("onImeShow"),
                    jni_sig!("(Z)V"),
                    &[multiline],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn on_ime_hide(&self) {
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                env.call_method(callback_ref(), jni_str!("onImeHide"), jni_sig!("()V"), &[])?;
                Ok(())
            })
            .unwrap();
    }

    fn on_media_session_metadata(&self, title: String, artist: String, album: String) {
        info!("on_media_session_metadata");
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let Ok(title) = new_string_as_jvalue(env, &title) else {
                    return Ok(());
                };

                let Ok(artist) = new_string_as_jvalue(env, &artist) else {
                    return Ok(());
                };

                let Ok(album) = new_string_as_jvalue(env, &album) else {
                    return Ok(());
                };

                env.call_method(
                    callback_ref(),
                    jni_str!("onMediaSessionMetadata"),
                    jni_sig!("(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V"),
                    &[(&title).into(), (&artist).into(), (&album).into()],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn on_media_session_playback_state_change(&self, state: MediaSessionPlaybackState) {
        info!("on_media_session_playback_state_change {:?}", state);
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let state = state as i32;
                let state = JValue::Int(state as jint);
                env.call_method(
                    callback_ref(),
                    jni_str!("onMediaSessionPlaybackStateChange"),
                    jni_sig!("(I)V"),
                    &[state],
                )?;
                Ok(())
            })
            .unwrap();
    }

    fn on_media_session_set_position_state(
        &self,
        duration: f64,
        position: f64,
        playback_rate: f64,
    ) {
        info!(
            "on_media_session_playback_state_change ({:?}, {:?}, {:?})",
            duration, position, playback_rate
        );
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let duration = JValue::Float(duration as jfloat);
                let position = JValue::Float(position as jfloat);
                let playback_rate = JValue::Float(playback_rate as jfloat);

                env.call_method(
                    callback_ref(),
                    jni_str!("onMediaSessionSetPositionState"),
                    jni_sig!("(FFF)V"),
                    &[duration, position, playback_rate],
                )?;
                Ok(())
            })
            .unwrap();
    }

    // KOTISATAMA-PATCH: JNI onOpenExternalResource → Android DownloadManager — JNI下载回调。
    fn on_open_external_resource(
        &self,
        url: String,
        mime_type: Option<String>,
        filename: Option<String>,
    ) {
        self.jvm
            .attach_current_thread(|env| -> Result<(), Error> {
                let Ok(url) = new_string_as_jvalue(env, &url) else {
                    return Ok(());
                };
                let mime_type = mime_type.unwrap_or_default();
                let filename = filename.unwrap_or_default();
                let Ok(mime_type) = new_string_as_jvalue(env, &mime_type) else {
                    return Ok(());
                };
                let Ok(filename) = new_string_as_jvalue(env, &filename) else {
                    return Ok(());
                };
                env.call_method(
                    callback_ref(),
                    jni_str!("onOpenExternalResource"),
                    jni_sig!("(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)V"),
                    &[(&url).into(), (&mime_type).into(), (&filename).into()],
                )?;
                Ok(())
            })
            .unwrap();
    }

    // KOTISATAMA-PATCH: <select> → Kotlin AlertDialog — <select>交给Kotlin对话框。
    fn on_show_select_element(&self, _webview_id: WebViewId, prompt: SelectElement) {
        embedder_prompts::show_select_element(prompt);
    }

    // KOTISATAMA-PATCH: context menu → Kotlin AlertDialog — 右键菜单交给Kotlin对话框。
    fn on_show_context_menu(&self, _webview_id: WebViewId, menu: ContextMenu) {
        embedder_prompts::show_context_menu(menu);
    }

    fn on_panic(&self, _reason: String, _backtrace: Option<String>) {}
}

unsafe extern "C" {
    pub fn __android_log_write(prio: c_int, tag: *const c_char, text: *const c_char) -> c_int;
}

fn throw(env: &mut Env, err: &JNIStr) {
    if let Err(e) = env.throw(err) {
        warn!(
            "Failed to throw Java exception: `{}`. Exception was: `{}`",
            e, err
        );
    }
}

fn new_string_as_jvalue<'local>(
    env: &mut Env<'local>,
    input_string: &str,
) -> Result<JValueOwned<'local>, &'static str> {
    let jstring = match env.new_string(input_string) {
        Ok(jstring) => jstring,
        Err(_) => {
            throw(env, jni_str!("Couldn't create Java string"));
            return Err("Couldn't create Java string");
        },
    };
    Ok(JValueOwned::from(jstring))
}

fn jni_coordinate_to_rust_viewport_rect<'local>(
    env: &mut Env<'local>,
    size: &JObject<'local>,
) -> Result<Rect<i32, DevicePixel>, Error> {
    let width = env
        .call_method(size, jni_str!("getWidth"), jni_sig!("()I"), &[])?
        .i()?;
    let height = env
        .call_method(size, jni_str!("getHeight"), jni_sig!("()I"), &[])?
        .i()?;

    Ok(Rect::new(Point2D::origin(), Size2D::new(width, height)))
}

fn set_default_config_dir<'local>(
    env: &mut Env<'local>,
    context: &JObject<'local>,
) -> Result<(), Error> {
    let files_dir = env
        .call_method(
            context,
            jni_str!("getFilesDir"),
            jni_sig!("()Ljava/io/File;"),
            &[],
        )?
        .l()?;
    let path = env
        .call_method(
            &files_dir,
            jni_str!("getAbsolutePath"),
            jni_sig!("()Ljava/lang/String;"),
            &[],
        )?
        .l()?;
    let path = JString::cast_local(env, path)?.try_to_string(env)?;

    let config_dir = PathBuf::from(path).join("servo");
    if let Err(error) = std::fs::create_dir_all(&config_dir) {
        error!("Failed to create config directory at {config_dir:?}: {error:?}");
    }
    debug!("Default config dir: {config_dir:?}");
    let _ = crate::prefs::DEFAULT_CONFIG_DIR
        .set(config_dir)
        .inspect_err(|path| warn!("Default config dir was already set to {path:?}"));
    Ok(())
}

fn display_and_window_handle(
    env: &mut Env<'_>,
    surface: &JObject<'_>,
) -> (RawDisplayHandle, RawWindowHandle) {
    let native_window = unsafe { ANativeWindow_fromSurface(env.get_raw(), surface.as_raw()) };
    let native_window = NonNull::new(native_window).expect("Could not get Android window");
    (
        RawDisplayHandle::Android(AndroidDisplayHandle::new()),
        RawWindowHandle::AndroidNdk(AndroidNdkWindowHandle::new(native_window)),
    )
}
