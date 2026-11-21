package com.versionlens.jetbrains

import com.google.gson.JsonObject
import com.google.gson.JsonParser
import com.intellij.credentialStore.CredentialAttributes
import com.intellij.credentialStore.generateServiceName
import com.intellij.ide.passwordSafe.PasswordSafe
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.PersistentStateComponent
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.State
import com.intellij.openapi.components.Storage
import com.intellij.openapi.options.BoundConfigurable
import com.intellij.openapi.project.ProjectManager
import com.intellij.openapi.ui.DialogPanel
import com.intellij.ui.dsl.builder.bindIntText
import com.intellij.ui.dsl.builder.bindSelected
import com.intellij.ui.dsl.builder.bindText
import com.intellij.ui.dsl.builder.panel
import com.intellij.ui.dsl.builder.rows
import org.eclipse.lsp4j.DidChangeConfigurationParams

/**
 * The `versionlens` settings the server reads, as in VS Code; the GitHub token is in PasswordSafe.
 */
@Service(Service.Level.APP)
@State(name = "VersionLens", storages = [Storage("versionlens.xml")])
internal class Settings : PersistentStateComponent<Settings> {
    var serverPath: String = ""
    var enabled: Boolean = true
    var codeLens: Boolean = true
    var inlayHints: Boolean = true
    var prereleases: Boolean = false
    var vulnerabilities: Boolean = true
    var timeoutMs: Int = 10_000
    var concurrency: Int = 32
    var cacheMinutes: Int = 60
    var strictSsl: Boolean = true
    var registries: String = "{}"

    override fun getState(): Settings = this

    override fun loadState(state: Settings) {
        serverPath = state.serverPath
        enabled = state.enabled
        codeLens = state.codeLens
        inlayHints = state.inlayHints
        prereleases = state.prereleases
        vulnerabilities = state.vulnerabilities
        timeoutMs = state.timeoutMs
        concurrency = state.concurrency
        cacheMinutes = state.cacheMinutes
        strictSsl = state.strictSsl
        registries = state.registries
    }

    var githubToken: String
        get() = PasswordSafe.instance.getPassword(TOKEN).orEmpty()
        set(value) = PasswordSafe.instance.setPassword(TOKEN, value.ifBlank { null })

    /** The `versionlens` section sent to the server. */
    fun json(): JsonObject =
        JsonObject().apply {
            addProperty("enabled", enabled)
            addProperty("codeLens", codeLens)
            addProperty("inlayHints", inlayHints)
            addProperty("prereleases", prereleases)
            addProperty("vulnerabilities", vulnerabilities)
            addProperty("timeoutMs", timeoutMs)
            addProperty("concurrency", concurrency)
            addProperty("cacheMinutes", cacheMinutes)
            addProperty("strictSsl", strictSsl)
            githubToken.takeIf { it.isNotEmpty() }?.let { addProperty("githubToken", it) }
            add("registries", registryMap(registries))
        }

    companion object {
        private val TOKEN = CredentialAttributes(generateServiceName("VersionLens", "githubToken"))

        /** The application settings. */
        fun get(): Settings = ApplicationManager.getApplication().getService(Settings::class.java)

        /** Parses the registries setting, a JSON object of registry id to URL template. */
        fun registryMap(text: String): JsonObject =
            JsonParser.parseString(text.ifBlank { "{}" }).asJsonObject
    }
}

/** Settings | Tools | VersionLens. */
internal class SettingsPage : BoundConfigurable("VersionLens") {
    private val settings = Settings.get()
    private var token = settings.githubToken

    override fun createPanel(): DialogPanel = panel {
        row("Server path:") {
            textField().bindText(settings::serverPath).comment("Empty uses the bundled server.")
        }
        row { checkBox("Enabled").bindSelected(settings::enabled) }
        row { checkBox("Code lenses").bindSelected(settings::codeLens) }
        row { checkBox("Inlay hints").bindSelected(settings::inlayHints) }
        row { checkBox("Suggest prereleases").bindSelected(settings::prereleases) }
        row { checkBox("Check vulnerabilities with OSV").bindSelected(settings::vulnerabilities) }
        row { checkBox("Verify TLS certificates").bindSelected(settings::strictSsl) }
        row("Request timeout (ms):") { intTextField(500..600_000).bindIntText(settings::timeoutMs) }
        row("Concurrent requests:") { intTextField(1..256).bindIntText(settings::concurrency) }
        row("Cache (minutes):") { intTextField(0..10_080).bindIntText(settings::cacheMinutes) }
        row("GitHub token:") {
            passwordField().bindText(::token).comment("Empty uses GITHUB_TOKEN or GH_TOKEN.")
        }
        row("Registries:") {
            textArea()
                .rows(3)
                .bindText(settings::registries)
                .comment("""JSON: {"npm": "https://npm.example.com/{name}"}""")
                .validationOnApply {
                    runCatching { Settings.registryMap(it.text) }
                        .exceptionOrNull()
                        ?.let { error("Registries must be a JSON object: ${it.message}") }
                }
        }
    }

    override fun apply() {
        val path = settings.serverPath
        super.apply()
        settings.githubToken = token
        for (project in ProjectManager.getInstance().openProjects) {
            if (settings.serverPath != path) {
                Client.restart(project)
            } else {
                Client.running(project).forEach { client ->
                    client.sendNotification {
                        it.workspaceService.didChangeConfiguration(
                            DidChangeConfigurationParams(JsonObject()),
                        )
                    }
                }
            }
        }
    }
}
