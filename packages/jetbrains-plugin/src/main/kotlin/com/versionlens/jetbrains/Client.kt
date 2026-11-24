package com.versionlens.jetbrains

import com.google.gson.JsonObject
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.ide.ActivityTracker
import com.intellij.notification.NotificationGroupManager
import com.intellij.notification.NotificationType
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.Lsp4jClient
import com.intellij.platform.lsp.api.LspClient
import com.intellij.platform.lsp.api.LspClientManager
import com.intellij.platform.lsp.api.LspIntegrationProvider
import com.intellij.platform.lsp.api.LspServerNotificationsHandler
import com.intellij.platform.lsp.api.ProjectWideLspClientDescriptor
import java.util.concurrent.atomic.AtomicBoolean
import org.eclipse.lsp4j.ConfigurationItem
import org.eclipse.lsp4j.DidChangeConfigurationParams
import org.eclipse.lsp4j.MessageParams
import org.eclipse.lsp4j.jsonrpc.services.JsonNotification

/** Starts versionlens-lsp when the project opens a manifest it reads. */
internal class Provider : LspIntegrationProvider {
    override fun fileOpened(
        project: Project,
        file: VirtualFile,
        clientStarter: LspIntegrationProvider.LspClientStarter,
    ) {
        if (file.isInLocalFileSystem && Client.supports(project, file)) {
            clientStarter.ensureClientStarted(Client(project))
        }
    }
}

/** The LSP client that also receives the server's custom `versionlens/status` notification. */
internal class StatusClient(private val project: Project, handler: LspServerNotificationsHandler) :
    Lsp4jClient(
        // The server's window/logMessage lines go to the VersionLens tool window.
        object : LspServerNotificationsHandler by handler {
            override fun logMessage(params: MessageParams) {
                Log.of(project).add(params.message)
                handler.logMessage(params)
            }
        },
    ) {
    /** Caches the status and refreshes the toolbar actions that show it. */
    @JsonNotification("versionlens/status")
    fun status(params: JsonObject) {
        Status.parse(params)?.let(StatusCache.of(project)::put)
        ActivityTracker.getInstance().inc()
    }
}

/** One versionlens-lsp process per project, configured from [Settings]. */
internal class Client(project: Project) : ProjectWideLspClientDescriptor(project, "VersionLens") {
    override fun isSupportedFile(file: VirtualFile): Boolean = supports(project, file)

    override fun createCommandLine(): GeneralCommandLine {
        val path = Server.path() ?: error("versionlens-lsp not found")
        Log.of(project).add("starting $path")
        return GeneralCommandLine(path, "--stdio")
    }

    override fun getWorkspaceConfiguration(item: ConfigurationItem): Any? =
        if (item.section == "versionlens") Settings.get().json() else null

    // The server's window/logMessage lines go to the VersionLens tool window, and its
    // versionlens/status notifications to the status cache the toolbar reads.
    override fun createLsp4jClient(handler: LspServerNotificationsHandler): Lsp4jClient =
        StatusClient(project, handler)

    companion object {
        private val reported = AtomicBoolean()

        /** Whether the server reads [file]; a missing server is reported once per session. */
        fun supports(project: Project, file: VirtualFile): Boolean {
            val path = Server.path()
            return try {
                    path != null && Server.matches(Server.patterns(path), file.path)
                } catch (e: Exception) {
                    report(project, "cannot run $path: ${e.message}")
                    false
                }
                .also {
                    if (path == null)
                        report(
                            project,
                            "versionlens-lsp not found; set its path in Settings | Tools | VersionLens",
                        )
                }
        }

        private fun report(project: Project, message: String) {
            Log.of(project).add(message)
            if (reported.compareAndSet(false, true)) {
                NotificationGroupManager.getInstance()
                    .getNotificationGroup("VersionLens")
                    .createNotification(message, NotificationType.ERROR)
                    .notify(project)
            }
        }

        /** The newest server status of [file], or null before the server reported it. */
        fun statusOf(project: Project, file: VirtualFile): Status? {
            val client = running(project).firstOrNull() ?: return null
            return StatusCache.of(project).get(client.descriptor.getFileUri(file))
        }

        /** The running clients of [project]. */
        fun running(project: Project): Collection<LspClient> =
            LspClientManager.getInstance(project).getClients(Provider::class.java)

        /** Tells the servers of [project] that [Settings] changed; each pulls the new section. */
        fun reconfigure(project: Project) {
            running(project).forEach { client ->
                client.sendNotification {
                    it.workspaceService.didChangeConfiguration(
                        DidChangeConfigurationParams(JsonObject()),
                    )
                }
            }
        }

        /** Restarts the server in [project], asking it for its file types again. */
        fun restart(project: Project) {
            Server.reset()
            StatusCache.of(project).clear()
            reported.set(false)
            LspClientManager.getInstance(project)
                .stopAndRestartClientsIfNeeded(Provider::class.java)
        }
    }
}
