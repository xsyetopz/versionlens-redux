package com.versionlens.jetbrains

import com.google.gson.JsonArray
import com.google.gson.JsonObject
import com.google.gson.JsonParser

/** The plain-value settings, before secrets are merged in. */
internal data class Values(
    val enabled: Boolean = true,
    val codeLens: Boolean = true,
    val inlayHints: Boolean = true,
    val prereleases: Boolean = false,
    val vulnerabilities: Boolean = true,
    val timeoutMs: Int = 10_000,
    val concurrency: Int = 32,
    val cacheMinutes: Int = 60,
    val strictSsl: Boolean = true,
    val registries: String = "",
    val install: String = "",
    val files: String = "",
    val stats: Boolean = false,
    val updateSeverity: String = "info",
    val prereleaseTags: String = "",
    val fileTypes: String = "",
    val workspaceExclude: String = "",
    val indicators: String = "",
    val providers: String = "",
)

/** Builds the `versionlens` configuration section the server reads. */
internal object Section {
    /**
     * Parses `key=value` lines into an ordered map. Blank lines and `#` comments are skipped, the
     * first `=` splits key from value, and a later duplicate key wins. A JSON object, the format an
     * earlier version stored the registries in, is read as is. A line without `=` or with an empty
     * key throws [IllegalArgumentException], naming the line.
     */
    fun parseMap(text: String): Map<String, String> {
        val trimmed = text.trim()
        if (trimmed.startsWith("{")) {
            val json = JsonParser.parseString(trimmed).asJsonObject
            return json.entrySet().associate { it.key to it.value.asString }
        }
        val map = linkedMapOf<String, String>()
        text.lineSequence().forEachIndexed { index, raw ->
            val line = raw.trim()
            if (line.isEmpty() || line.startsWith("#")) return@forEachIndexed
            val split = line.indexOf('=')
            require(split > 0) { "line ${index + 1} is not key=value: $line" }
            map[line.substring(0, split).trim()] = line.substring(split + 1).trim()
        }
        return map
    }

    /** The authorization lines [text] with [url] sending [header], replacing an earlier entry. */
    fun withEntry(text: String, url: String, header: String): String =
        lines(parseMap(text) + (url to header))

    /** The authorization lines [text] without the entries of [urls]. */
    fun without(text: String, urls: Collection<String>): String =
        lines(parseMap(text) - urls.toSet())

    private fun lines(map: Map<String, String>): String =
        map.entries.joinToString("\n") { "${it.key}=${it.value}" }

    /** Merges the plain [values] with the secrets from PasswordSafe into the server section. */
    fun build(values: Values, githubToken: String, authorization: String): JsonObject =
        JsonObject().apply {
            addProperty("enabled", values.enabled)
            addProperty("codeLens", values.codeLens)
            addProperty("inlayHints", values.inlayHints)
            addProperty("prereleases", values.prereleases)
            addProperty("vulnerabilities", values.vulnerabilities)
            addProperty("timeoutMs", values.timeoutMs)
            addProperty("concurrency", values.concurrency)
            addProperty("cacheMinutes", values.cacheMinutes)
            addProperty("strictSsl", values.strictSsl)
            githubToken.takeIf { it.isNotBlank() }?.let { addProperty("githubToken", it) }
            add("registries", map(values.registries))
            add("authorization", map(authorization))
            add("install", map(values.install))
            add("files", map(values.files))
            addProperty("stats", values.stats)
            addProperty("updateSeverity", values.updateSeverity)
            add("prereleaseTags", list(values.prereleaseTags))
            add("fileTypes", list(values.fileTypes))
            add("workspaceExclude", list(values.workspaceExclude))
            add("indicators", map(values.indicators))
            add("providers", providers(values.providers))
        }

    /**
     * Parses `registry.setting=value` lines into the per-registry overrides: `cacheMinutes` takes a
     * whole number, `strictSsl` true or false, and `prereleaseTags` a comma-separated list. Throws
     * [IllegalArgumentException] naming the line that breaks one of these.
     */
    fun providers(text: String): JsonObject =
        JsonObject().apply {
            parseMap(text).forEach { (key, value) ->
                val registry = key.substringBeforeLast('.', "")
                require(registry.isNotEmpty()) { "$key is not registry.setting" }
                val entry = getAsJsonObject(registry) ?: JsonObject().also { add(registry, it) }
                when (val setting = key.substringAfterLast('.')) {
                    "cacheMinutes" ->
                        entry.addProperty(
                            setting,
                            requireNotNull(value.toIntOrNull()?.takeIf { it >= 0 }) {
                                "$key needs a whole number of minutes"
                            },
                        )
                    "strictSsl" ->
                        entry.addProperty(
                            setting,
                            requireNotNull(value.toBooleanStrictOrNull()) {
                                "$key needs true or false"
                            },
                        )
                    "prereleaseTags" -> entry.add(setting, list(value))
                    else ->
                        throw IllegalArgumentException(
                            "$key: use cacheMinutes, strictSsl, or prereleaseTags",
                        )
                }
            }
        }

    /** Splits a comma- or newline-separated list, dropping blanks. */
    fun parseList(text: String): List<String> =
        text.split(',', '\n').map { it.trim() }.filter { it.isNotEmpty() }

    private fun list(text: String): JsonArray =
        JsonArray().apply { parseList(text).forEach { add(it) } }

    private fun map(text: String): JsonObject =
        JsonObject().apply { parseMap(text).forEach { (key, value) -> addProperty(key, value) } }
}
