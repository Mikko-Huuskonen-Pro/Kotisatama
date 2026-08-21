/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// KOTISATAMA-PATCH: GL-kiihdytetty video Android EGL:stä — 从Android EGL初始化GL加速视频
use std::cell::RefMut;

use surfman::{Context, Device};

pub(crate) fn setup_gl_accelerated_media(device: RefMut<'_, Device>, context: RefMut<'_, Context>) {
    use servo::{MediaGlContext, MediaNativeDisplay, Servo};

    let api = api(&device, &context);
    let gl_context = MediaGlContext::Egl(device.native_context(&context).egl_context as usize);
    let display = MediaNativeDisplay::Egl(device.native_device().0 as usize);
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
