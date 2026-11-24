package com.versionlens.jetbrains

import com.google.gson.JsonObject
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
import com.intellij.ui.dsl.builder.AlignX
import com.intellij.ui.dsl.builder.Panel
import com.intellij.ui.dsl.builder.bindIntText
import com.intellij.ui.dsl.builder.bindItem
import com.intellij.ui.dsl.builder.bindSelected
import com.intellij.ui.dsl.builder.bindText
import com.intellij.ui.dsl.builder.panel
import com.intellij.ui.dsl.builder.rows
import com.intellij.ui.dsl.builder.toNullableProperty
import kotlin.reflect.KMutableProperty0

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
    var registries: String = ""
    var install: String = ""
    var files: String = ""
    var stats: Boolean = false
    var updateSeverity: String = "info"
    var prereleaseTags: String = ""
    var fileTypes: String = ""
    var workspaceExclude: String = ""
    var indicators: String = ""
    var providers: String = ""

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
        install = state.install
        files = state.files
        stats = state.stats
        updateSeverity = state.updateSeverity
        prereleaseTags = state.prereleaseTags
        fileTypes = state.fileTypes
        workspaceExclude = state.workspaceExclude
        indicators = state.indicators
        providers = state.providers
    }

    var githubToken: String
        get() = PasswordSafe.instance.getPassword(TOKEN).orEmpty()
        set(value) = PasswordSafe.instance.setPassword(TOKEN, value.ifBlank { null })

    var authorization: String
        get() = PasswordSafe.instance.getPassword(AUTHORIZATION).orEmpty()
        set(value) = PasswordSafe.instance.setPassword(AUTHORIZATION, value.ifBlank { null })

    /** The `versionlens` section sent to the server, with the secrets merged in. */
    fun json(): JsonObject =
        Section.build(
            Values(
                enabled,
                codeLens,
                inlayHints,
                prereleases,
                vulnerabilities,
                timeoutMs,
                concurrency,
                cacheMinutes,
                strictSsl,
                registries,
                install,
                files,
                stats,
                updateSeverity,
                prereleaseTags,
                fileTypes,
                workspaceExclude,
                indicators,
                providers,
            ),
            githubToken,
            authorization,
        )

    companion object {
        private val TOKEN = CredentialAttributes(generateServiceName("VersionLens", "githubToken"))
        private val AUTHORIZATION =
            CredentialAttributes(generateServiceName("VersionLens", "authorization"))

        /** The application settings. */
        fun get(): Settings = ApplicationManager.getApplication().getService(Settings::class.java)
    }
}

/** Settings | Tools | VersionLens. */
internal class SettingsPage : BoundConfigurable("VersionLens") {
    private val settings = Settings.get()
    private var token = settings.githubToken
    private var authorization = settings.authorization

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
        row { checkBox("Summary lens above each manifest").bindSelected(settings::stats) }
        row("Outdated dependency severity:") {
            comboBox(SEVERITIES).bindItem(settings::updateSeverity.toNullableProperty())
        }
        row("Prerelease tags:") {
            textField()
                .bindText(settings::prereleaseTags)
                .comment("Comma-separated, for example rc, beta; empty allows every tag.")
        }
        row("File types:") {
            textField()
                .bindText(settings::fileTypes)
                .comment("Comma-separated file type or registry ids; empty reads every file type.")
        }
        row("Workspace exclude:") {
            textField()
                .bindText(settings::workspaceExclude)
                .comment("Comma-separated globs a workspace update skips, for example examples/**.")
        }
        row("Request timeout (ms):") { intTextField(500..600_000).bindIntText(settings::timeoutMs) }
        row("Concurrent requests:") { intTextField(1..256).bindIntText(settings::concurrency) }
        row("Cache (minutes):") { intTextField(0..10_080).bindIntText(settings::cacheMinutes) }
        row("GitHub token:") {
            passwordField().bindText(::token).comment("Empty uses GITHUB_TOKEN or GH_TOKEN.")
        }
        maps()
    }

    private fun Panel.maps() {
        lines("Registries:", settings::registries, "registry id=URL template, one per line")
        lines(
            "Per registry:",
            settings::providers,
            "registry id.setting=value; settings: cacheMinutes, strictSsl, prereleaseTags",
            Section::providers,
        )
        lines(
            "Authorization:",
            ::authorization,
            "URL prefix=Authorization header value, one per line; stored in PasswordSafe",
        )
        lines(
            "Install commands:",
            settings::install,
            "file type or registry id=shell command run on save when dependencies change",
        )
        lines("Extra files:", settings::files, "glob=file type id, for example *.deps=npm")
        lines(
            "Indicators:",
            settings::indicators,
            "state=glyph; states: latest, update, checking, failed, unreadable, vulnerable, newer, satisfies, noMatch",
        )
    }

    private fun Panel.lines(
        label: String,
        property: KMutableProperty0<String>,
        comment: String,
        parse: (String) -> Any = Section::parseMap,
    ) =
        row(label) {
            textArea()
                .rows(3)
                .align(AlignX.FILL)
                .bindText(property)
                .comment(comment)
                .validationOnApply {
                    runCatching { parse(it.text) }
                        .exceptionOrNull()
                        ?.let { error("Use key=value lines: ${it.message}") }
                }
        }

    private companion object {
        val SEVERITIES = listOf("off", "hint", "info", "warning", "error")
    }

    override fun apply() {
        val path = settings.serverPath
        super.apply()
        settings.githubToken = token
        settings.authorization = authorization
        for (project in ProjectManager.getInstance().openProjects) {
            if (settings.serverPath != path) Client.restart(project)
            else Client.reconfigure(project)
        }
    }
}
