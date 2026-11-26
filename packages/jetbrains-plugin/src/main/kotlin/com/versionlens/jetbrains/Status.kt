package com.versionlens.jetbrains

import com.google.gson.JsonObject
import com.intellij.openapi.components.Service
import com.intellij.openapi.project.Project
import java.util.concurrent.ConcurrentHashMap

/** The `versionlens/status` payload the server sends for one open manifest. */
internal data class Status(
    val uri: String,
    val enabled: Boolean,
    val prereleases: Boolean,
    val busy: Boolean,
    val dependencies: Int,
    val outdated: Int,
    val failed: Int,
    val vulnerable: Int,
    val sortable: Boolean,
    val install: String?,
) {
    companion object {
        /** Parses the notification params; a payload without a `uri` is rejected with null. */
        fun parse(json: JsonObject): Status? {
            val uri = json.text("uri") ?: return null
            return Status(
                uri = uri,
                enabled = json.flag("enabled"),
                prereleases = json.flag("prereleases"),
                busy = json.flag("busy"),
                dependencies = json.count("dependencies"),
                outdated = json.count("outdated"),
                failed = json.count("failed"),
                vulnerable = json.count("vulnerable"),
                sortable = json.flag("sortable"),
                install = json.text("install"),
            )
        }

        private fun JsonObject.text(key: String): String? =
            get(key)?.takeIf { it.isJsonPrimitive }?.asString

        private fun JsonObject.flag(key: String): Boolean =
            get(key)?.takeIf { it.isJsonPrimitive }?.asBoolean ?: false

        private fun JsonObject.count(key: String): Int =
            get(key)?.takeIf { it.isJsonPrimitive }?.asInt ?: 0
    }
}

/** The latest [Status] of each manifest of one project, by file URI. */
@Service(Service.Level.PROJECT)
internal class StatusCache {
    private val statuses = ConcurrentHashMap<String, Status>()

    /** Remembers [status] as the newest for its file. */
    fun put(status: Status) {
        statuses[status.uri] = status
    }

    /** The newest status of the file at [uri], or null before the server reported it. */
    fun get(uri: String): Status? = statuses[uri]

    /** Forgets every status, as after a server restart. */
    fun clear() = statuses.clear()

    companion object {
        /** The cache of [project]. */
        fun of(project: Project): StatusCache = project.getService(StatusCache::class.java)
    }
}
