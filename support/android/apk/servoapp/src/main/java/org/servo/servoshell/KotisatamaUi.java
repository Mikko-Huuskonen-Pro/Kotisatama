/* -*- Mode: Java; c-basic-offset: 4; tab-width: 4; indent-tabs-mode: nil; -*-
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

package org.servo.servoshell;

import android.app.Activity;
import android.app.AlertDialog;
import android.text.InputType;
import android.view.View;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.RadioButton;
import android.widget.RadioGroup;
import android.widget.Toast;

import org.json.JSONArray;
import org.json.JSONObject;
import org.servo.servoview.ServoView;

/**
 * Kotisatama search and report UI for Android (mirrors desktop servoshell toolbar).
 */
public final class KotisatamaUi {
    private KotisatamaUi() {}

    public static boolean isLikelyUrl(String text) {
        if (text == null || text.isEmpty()) {
            return false;
        }
        String lower = text.toLowerCase();
        return lower.startsWith("http://")
                || lower.startsWith("https://")
                || lower.startsWith("about:")
                || lower.contains(".");
    }

    public static void handleUrlOrSearch(Activity activity, ServoView servoView, String text) {
        text = text.trim();
        if (text.isEmpty()) {
            return;
        }
        if (isLikelyUrl(text)) {
            servoView.loadUri(text);
            return;
        }
        showSearchResults(activity, servoView, text);
    }

    public static void showSearchResults(Activity activity, ServoView servoView, String query) {
        String json = servoView.kotisatamaSearch(query);
        try {
            JSONObject root = new JSONObject(json);
            String type = root.optString("type", "error");
            if ("hits".equals(type)) {
                JSONArray hits = root.getJSONArray("hits");
                if (hits.length() == 0) {
                    offerAvomeri(activity, servoView, query);
                    return;
                }
                String[] labels = new String[hits.length()];
                final String[] urls = new String[hits.length()];
                for (int i = 0; i < hits.length(); i++) {
                    JSONObject hit = hits.getJSONObject(i);
                    urls[i] = hit.getString("url");
                    labels[i] = hit.getString("title") + " — " + urls[i];
                }
                new AlertDialog.Builder(activity)
                        .setTitle("Kotisatama-haku: " + query)
                        .setItems(labels, (dialog, which) -> servoView.loadUri(urls[which]))
                        .setNegativeButton("Sulje", null)
                        .show();
            } else if ("no_results".equals(type)) {
                offerAvomeri(activity, servoView, query);
            } else {
                String message = root.optString("message", "Haku epäonnistui");
                Toast.makeText(activity, message, Toast.LENGTH_LONG).show();
            }
        } catch (Exception e) {
            Toast.makeText(activity, "Haku epäonnistui", Toast.LENGTH_LONG).show();
        }
    }

    private static void offerAvomeri(Activity activity, ServoView servoView, String query) {
        new AlertDialog.Builder(activity)
                .setTitle("Ei löydy kotisatamasta")
                .setMessage("Haluatko hakea avomereltä?")
                .setPositiveButton("Hae avomereltä", (dialog, which) -> {
                    String encoded = android.net.Uri.encode(query);
                    servoView.loadUri("https://www.startpage.com/search?q=" + encoded);
                })
                .setNegativeButton("Peruuta", null)
                .show();
    }

    public static void showReportDialog(Activity activity, ServoView servoView, String currentUrl) {
        if (!servoView.kotisatamaShouldShowReport(currentUrl)) {
            return;
        }

        LinearLayout layout = new LinearLayout(activity);
        layout.setOrientation(LinearLayout.VERTICAL);
        int padding = (int) (16 * activity.getResources().getDisplayMetrics().density);
        layout.setPadding(padding, padding, padding, padding);

        RadioGroup kindGroup = new RadioGroup(activity);
        RadioButton broken = new RadioButton(activity);
        broken.setText("Sivusto ei toimi");
        int brokenId = View.generateViewId();
        broken.setId(brokenId);
        RadioButton suggest = new RadioButton(activity);
        suggest.setText("Ehdota kotisatamaan");
        int suggestId = View.generateViewId();
        suggest.setId(suggestId);
        kindGroup.addView(broken);
        kindGroup.addView(suggest);

        boolean blockedPage = currentUrl != null && currentUrl.startsWith("data:text/html");
        if (blockedPage) {
            suggest.setChecked(true);
        } else {
            broken.setChecked(true);
        }

        EditText domainField = new EditText(activity);
        domainField.setHint("domain (esim. kela.fi)");
        domainField.setInputType(InputType.TYPE_CLASS_TEXT);
        domainField.setText(domainFromUrl(currentUrl));

        EditText messageField = new EditText(activity);
        messageField.setHint("Kuvaus (valinnainen)");
        messageField.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_MULTI_LINE);
        messageField.setMinLines(3);

        layout.addView(kindGroup);
        layout.addView(domainField);
        layout.addView(messageField);

        new AlertDialog.Builder(activity)
                .setTitle("Ilmoita")
                .setMessage("Anonyymi raportti — ei käyttäjätunnistetta.")
                .setView(layout)
                .setPositiveButton("Lähetä", (dialog, which) -> {
                    String kind = kindGroup.getCheckedRadioButtonId() == suggestId
                            ? "suggest_site"
                            : "site_broken";
                    String domain = domainField.getText().toString().trim();
                    String message = messageField.getText().toString().trim();
                    String contextUrl = currentUrl != null ? currentUrl : "";
                    String error = servoView.kotisatamaSubmitReport(kind, domain, message, contextUrl);
                    if (error == null || error.isEmpty()) {
                        Toast.makeText(activity, "Raportti lähetetty", Toast.LENGTH_SHORT).show();
                    } else {
                        Toast.makeText(activity, error, Toast.LENGTH_LONG).show();
                    }
                })
                .setNegativeButton("Peruuta", null)
                .show();
    }

    private static String domainFromUrl(String url) {
        if (url == null || url.isEmpty()) {
            return "";
        }
        try {
            android.net.Uri uri = android.net.Uri.parse(url);
            String host = uri.getHost();
            return host != null ? host : url;
        } catch (Exception e) {
            return url;
        }
    }
}
