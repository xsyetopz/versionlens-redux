package com.versionlens.jetbrains

import com.google.gson.JsonParser
import com.intellij.execution.ExecutionException
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.configurations.PathEnvironmentVariableUtil
import com.intellij.execution.util.ExecUtil
import com.intellij.ide.plugins.PluginManagerCore
import com.intellij.openapi.extensions.PluginId
import java.io.File
import java.util.Locale

/** Finds the versionlens-lsp binary and the manifests it reads. */
internal object Server {
    private val windows = System.getProperty("os.name").startsWith("Windows", ignoreCase = true)
    private val binary = if (windows) "versionlens-lsp.exe" else "versionlens-lsp"
    @Volatile private var cached: Pair<String, List<Regex>>? = null

    /** The Rust target triple of a release build for this machine, or null. */
    fun triple(
        os: String = System.getProperty("os.name"),
        arch: String = System.getProperty("os.arch"),
        musl: Boolean = File("/etc/alpine-release").exists(),
    ): String? {
        val cpu =
            when (arch.lowercase(Locale.ROOT)) {
                "amd64",
                "x86_64" -> "x86_64"
                "aarch64",
                "arm64" -> "aarch64"
                "arm",
                "armv7",
                "armv7l" -> "armv7"
                else -> return null
            }
        return when {
            os.startsWith("Windows", ignoreCase = true) && cpu != "armv7" -> "$cpu-pc-windows-msvc"
            os.startsWith("Mac", ignoreCase = true) && cpu != "armv7" -> "$cpu-apple-darwin"
            os.startsWith("Linux", ignoreCase = true) && cpu == "armv7" ->
                if (musl) null else "armv7-unknown-linux-gnueabihf"
            os.startsWith("Linux", ignoreCase = true) ->
                "$cpu-unknown-linux-${if (musl) "musl" else "gnu"}"
            else -> null
        }
    }

    /**
     * The server path: `VERSIONLENS_LSP`, the configured path, the bundled release, then `PATH`.
     */
    fun path(): String? {
        System.getenv("VERSIONLENS_LSP")
            ?.takeIf { it.isNotBlank() }
            ?.let {
                return it
            }
        Settings.get()
            .serverPath
            .takeIf { it.isNotBlank() }
            ?.let {
                return it
            }
        bundled()?.let {
            return it
        }
        return PathEnvironmentVariableUtil.findInPath(binary)?.path
    }

    private fun bundled(): String? {
        val root = PluginManagerCore.getPlugin(PluginId.getId(PLUGIN_ID))?.pluginPath ?: return null
        val file = root.resolve("bin").resolve(triple() ?: return null).resolve(binary).toFile()
        if (!file.isFile) return null
        // Plugin archives can lose the executable bit when the IDE unpacks them.
        if (!windows && !file.canExecute()) file.setExecutable(true)
        return file.path
    }

    /**
     * Converts a server glob, where `*` stays within one path segment, to a regex over `/` paths.
     */
    fun pattern(glob: String): Regex =
        Regex(
            "(^|/)" + glob.split("*").joinToString("[^/]*") { Regex.escape(it) } + "$",
            RegexOption.IGNORE_CASE,
        )

    /** Patterns for every manifest the server at [path] reads, asked from it once per path. */
    fun patterns(path: String): List<Regex> {
        cached
            ?.takeIf { it.first == path }
            ?.let {
                return it.second
            }
        val output = ExecUtil.execAndGetOutput(GeneralCommandLine(path, "--file-types"), 10_000)
        if (output.exitCode != 0) {
            throw ExecutionException(
                "$path --file-types exited with ${output.exitCode}: ${output.stderr}",
            )
        }
        val globs = JsonParser.parseString(output.stdout).asJsonObject.getAsJsonArray("globs")
        val list = globs.map { pattern(it.asString) }
        cached = path to list
        return list
    }

    /** Forgets the file patterns, so the next start asks the server again. */
    fun reset() {
        cached = null
    }

    /** Whether [patterns] match the file at [path]. */
    fun matches(patterns: List<Regex>, path: String): Boolean {
        val normalized = path.replace(File.separatorChar, '/')
        return patterns.any { it.containsMatchIn(normalized) }
    }

    const val PLUGIN_ID: String = "com.versionlens.jetbrains"
}
