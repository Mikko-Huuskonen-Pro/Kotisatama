/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// KOTISATAMA-PATCH: GL-kiihdytetty video Android EGL:stä — 从Android EGL初始化GL加速视频
use std::cell::RefMut;

use surfman::{Context, Device};

pub(crate) fn setup_gl_accelerated_media(device: RefMut<'_, Device>, context: RefMut<'_, Context>) {
    use servo::{MediaGlApi, MediaGlContext, MediaNativeDisplay, Servo};
    use surfman::multi::connection::NativeConnection;
    use surfman::multi::context::NativeContext;

    let api = api(&device, &context);
    let (gl_context, display) = match device.native_context(&context) {
        NativeContext::Default(NativeContext::Default(native_context)) => (
            MediaGlContext::Egl(native_context.egl_context as usize),
            match device.connection().native_connection() {
                surfman::NativeConnection::Default(NativeConnection::Default(connection)) => {
                    MediaNativeDisplay::Egl(connection.0 as usize)
                },
                _ => MediaNativeDisplay::Unknown,
            },
        ),
        NativeContext::Default(NativeContext::Alternate(native_context)) => (
            MediaGlContext::Egl(native_context.egl_context as usize),
            MediaNativeDisplay::Unknown,
        ),
        NativeContext::Alternate(_) => (MediaGlContext::Unknown, MediaNativeDisplay::Unknown),
    };

    if matches!(display, MediaNativeDisplay::Unknown) || matches!(gl_context, MediaGlContext::Unknown)
    {
        log::warn!("Could not extract EGL handles for GL accelerated media on Android");
        return;
    }

    Servo::initialize_gl_accelerated_media(display, api, gl_context);
}

fn api(device: &RefMut<Device>, context: &RefMut<Context>) -> servo::MediaGlApi {
    use servo::MediaGlApi;
    use surfman::GLApi;

    let descriptor = device.context_descriptor(context);
    let attributes = device.context_descriptor_attributes(&descriptor);
    let major = attributes.version.major;
    let minor = attributes.version.minor;
    match device.connection().gl_api() {
        GLApi::GL if major >= 3 && minor >= 2 => MediaGlApi::OpenGL3,
        GLApi::GL => MediaGlApi::OpenGL,
        GLApi::GLES if major > 1 => MediaGlApi::Gles2,
        GLApi::GLES => MediaGlApi::Gles1,
    }
}
