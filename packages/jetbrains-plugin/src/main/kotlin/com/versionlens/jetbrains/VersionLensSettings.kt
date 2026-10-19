package com.versionlens.jetbrains

import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage

/**
 * The VersionLens settings the language server reads with
 * `workspace/configuration`. They are a JSON object with the same keys as the
 * `versionlens` section of the VS Code settings, stored in `versionlens-redux.xml`.
 */
@Service(Service.Level.APP)
@State(name = "VersionLensSettings", storages = [Storage("versionlens-redux.xml")])
internal class VersionLensSettings : PersistentStateComponent<VersionLensSettings.State> {
    class State {
        var settingsJson: String = "{}"
    }

    private var current = State()

    override fun getState(): State = current

    override fun loadState(state: State) {
        current = state
    }
}
