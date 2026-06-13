/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.servo.servoshell;

import android.content.Context;
import android.content.res.AssetManager;
import android.system.ErrnoException;
import android.system.Os;
import android.util.Log;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;

/**
 * Extract Kotisatama bundled assets and configure environment before Servo init.
 */
public final class KotisatamaAssets {
    private static final String TAG = "KotisatamaAssets";
    private static final String ASSET_PREFIX = "kotisatama/";

    private KotisatamaAssets() {}

    public static void prepare(Context context) {
        File base = new File(context.getFilesDir(), "kotisatama");
        if (!base.exists() && !base.mkdirs()) {
            Log.e(TAG, "Failed to create kotisatama data directory");
            return;
        }

        File whitelist = extractAsset(context, ASSET_PREFIX + "whitelist.json", new File(base, "whitelist.json"));
        File documents = extractAsset(context, ASSET_PREFIX + "documents.json", new File(base, "documents.json"));
        File indexDump = extractAssetIfPresent(context, ASSET_PREFIX + "index.dump", new File(base, "index.dump"));
        File meilisearch = extractAssetIfPresent(context, ASSET_PREFIX + "bin/meilisearch", new File(base, "bin/meilisearch"));

        File dbDir = new File(base, "meilisearch-db");
        if (!dbDir.exists()) {
            dbDir.mkdirs();
        }

        try {
            Os.setenv("KOTISATAMA_DATA_DIR", base.getAbsolutePath(), true);
            if (whitelist != null) {
                Os.setenv("KOTISATAMA_WHITELIST_PATH", whitelist.getAbsolutePath(), true);
            }
            if (documents != null) {
                Os.setenv("KOTISATAMA_SEARCH_DOCUMENTS", documents.getAbsolutePath(), true);
            }
            if (indexDump != null) {
                Os.setenv("KOTISATAMA_INDEX_DUMP", indexDump.getAbsolutePath(), true);
            }
            Os.setenv("KOTISATAMA_MEILISEARCH_DB", dbDir.getAbsolutePath(), true);
            if (meilisearch != null) {
                meilisearch.setExecutable(true, false);
                Os.setenv("KOTISATAMA_MEILISEARCH_BIN", meilisearch.getAbsolutePath(), true);
            } else {
                Log.w(TAG, "Meilisearch binary not bundled — local search unavailable offline");
            }
        } catch (ErrnoException e) {
            Log.e(TAG, "Failed to set Kotisatama environment variables", e);
        }
    }

    private static File extractAsset(Context context, String assetPath, File dest) {
        if (dest.exists() && dest.length() > 0) {
            return dest;
        }
        File parent = dest.getParentFile();
        if (parent != null && !parent.exists()) {
            parent.mkdirs();
        }
        if (copyAsset(context.getAssets(), assetPath, dest)) {
            return dest;
        }
        Log.w(TAG, "Missing required asset: " + assetPath);
        return null;
    }

    private static File extractAssetIfPresent(Context context, String assetPath, File dest) {
        if (dest.exists() && dest.length() > 0) {
            return dest;
        }
        File parent = dest.getParentFile();
        if (parent != null && !parent.exists()) {
            parent.mkdirs();
        }
        if (copyAsset(context.getAssets(), assetPath, dest)) {
            return dest;
        }
        return null;
    }

    private static boolean copyAsset(AssetManager assets, String assetPath, File dest) {
        try (InputStream in = assets.open(assetPath);
             OutputStream out = new FileOutputStream(dest)) {
            byte[] buffer = new byte[8192];
            int read;
            while ((read = in.read(buffer)) != -1) {
                out.write(buffer, 0, read);
            }
            return true;
        } catch (IOException e) {
            return false;
        }
    }
}
