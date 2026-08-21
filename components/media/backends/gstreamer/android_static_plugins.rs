/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! KOTISATAMA-PATCH: Android GStreamer SDK käyttää staattisia plugin-.a-arkistoja — rekisteröidään gst_init:n jälkeen
//! — Android GStreamer SDK 使用静态插件 .a 归档，需在 gst_init 之后注册

#![cfg(target_os = "android")]

use glib_sys::gboolean;
use log::warn;

macro_rules! register_plugin {
    ($register_fn:ident) => {
        unsafe extern "C" {
            fn $register_fn() -> gboolean;
        }
        let ok = unsafe { $register_fn() };
        if ok == glib_sys::GFALSE {
            warn!(
                "GStreamer static plugin registration failed: {}",
                stringify!($register_fn)
            );
        }
    };
}

/// Rekisteröi Tier-1 HTML5-videoon tarvittavat staattiset pluginit (MP4/WebM + Android-audio/GL).
pub fn register_static_plugins() {
    // core + playback
    register_plugin!(gst_plugin_coreelements_register);
    register_plugin!(gst_plugin_typefindfunctions_register);
    register_plugin!(gst_plugin_playback_register);
    register_plugin!(gst_plugin_app_register);
    register_plugin!(gst_plugin_volume_register);
    register_plugin!(gst_plugin_autodetect_register);
    // demux / parse
    register_plugin!(gst_plugin_isomp4_register);
    register_plugin!(gst_plugin_matroska_register);
    register_plugin!(gst_plugin_id3demux_register);
    register_plugin!(gst_plugin_audioparsers_register);
    register_plugin!(gst_plugin_videoparsersbad_register);
    register_plugin!(gst_plugin_wavparse_register);
    // convert / filter
    register_plugin!(gst_plugin_audioconvert_register);
    register_plugin!(gst_plugin_audioresample_register);
    register_plugin!(gst_plugin_videoconvertscale_register);
    register_plugin!(gst_plugin_videofilter_register);
    register_plugin!(gst_plugin_deinterlace_register);
    register_plugin!(gst_plugin_interleave_register);
    // codecs (software + Android HW)
    register_plugin!(gst_plugin_libav_register);
    register_plugin!(gst_plugin_androidmedia_register);
    register_plugin!(gst_plugin_vpx_register);
    register_plugin!(gst_plugin_opus_register);
    register_plugin!(gst_plugin_vorbis_register);
    register_plugin!(gst_plugin_theora_register);
    register_plugin!(gst_plugin_ogg_register);
    // Android output + GL sink (glsinkbin)
    register_plugin!(gst_plugin_opensles_register);
    register_plugin!(gst_plugin_opengl_register);
    // Helpers for Tier-1 playback (ei WebRTC/ICE — M1)
    register_plugin!(gst_plugin_gio_register);
    register_plugin!(gst_plugin_audiofx_register);
    register_plugin!(gst_plugin_id3tag_register);
}
