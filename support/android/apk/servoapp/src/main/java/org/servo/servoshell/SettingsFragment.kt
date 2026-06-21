package org.servo.servoshell

import android.content.SharedPreferences
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.RadioGroup
import androidx.fragment.app.Fragment
import androidx.preference.PreferenceManager
import com.google.android.material.switchmaterial.SwitchMaterial
import androidx.core.content.edit

class SettingsFragment : Fragment() {
    private lateinit var preferences: SharedPreferences
    private lateinit var experimentalSwitch: SwitchMaterial
    private lateinit var animatingSwitch: SwitchMaterial
    private lateinit var languageGroup: RadioGroup

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        preferences = PreferenceManager.getDefaultSharedPreferences(requireContext())
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?,
    ): View = inflater.inflate(R.layout.fragment_settings, container, false)

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        experimentalSwitch = view.findViewById(R.id.experimental_switch)
        animatingSwitch = view.findViewById(R.id.animating_switch)
        languageGroup = view.findViewById(R.id.language_group)
        val experimentalContainer = view.findViewById<View>(R.id.experimental_container)
        val animatingContainer = view.findViewById<View>(R.id.animating_container)

        loadPreferences()
        loadLanguagePreference()

        languageGroup.setOnCheckedChangeListener { _, checkedId ->
            val value = when (checkedId) {
                R.id.language_fi -> "fi"
                R.id.language_sv -> "sv"
                else -> "auto"
            }
            if (saveLanguagePreference(value)) {
                requireActivity().recreate()
            }
        }

        experimentalContainer.setOnClickListener {
            val newValue = !experimentalSwitch.isChecked
            experimentalSwitch.isChecked = newValue
            savePreference("experimental", newValue)
        }

        animatingContainer.setOnClickListener {
            val newValue = !animatingSwitch.isChecked
            animatingSwitch.isChecked = newValue
            savePreference("animating_indicator", newValue)
        }

        experimentalSwitch.setOnCheckedChangeListener { buttonView, isChecked ->
            if (buttonView.isPressed) {
                savePreference("experimental", isChecked)
            }
        }

        animatingSwitch.setOnCheckedChangeListener { buttonView, isChecked ->
            if (buttonView.isPressed) {
                savePreference("animating_indicator", isChecked)
            }
        }
    }

    private fun loadPreferences() {
        val experimental = preferences.getBoolean("experimental", false)
        val animatingIndicator = preferences.getBoolean("animating_indicator", false)

        experimentalSwitch.isChecked = experimental
        animatingSwitch.isChecked = animatingIndicator
    }

    private fun savePreference(key: String, value: Boolean) {
        preferences.edit { putBoolean(key, value) }
    }

    private fun loadLanguagePreference() {
        val code = preferences.getString(LocaleHelper.PREF_KEY, "auto") ?: "auto"
        val checkedId = when (code) {
            "fi" -> R.id.language_fi
            "sv" -> R.id.language_sv
            else -> R.id.language_auto
        }
        languageGroup.check(checkedId)
    }

    private fun saveLanguagePreference(value: String): Boolean {
        val current = preferences.getString(LocaleHelper.PREF_KEY, "auto") ?: "auto"
        if (current == value) {
            return false
        }
        preferences.edit { putString(LocaleHelper.PREF_KEY, value) }
        return true
    }
}
