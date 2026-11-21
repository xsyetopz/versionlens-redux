package com.versionlens.jetbrains

import com.intellij.execution.configurations.GeneralCommandLine
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
import org.eclipse.lsp4j.MessageParams

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

    // The server's window/logMessage lines go to the VersionLens tool window.
    override fun createLsp4jClient(handler: LspServerNotificationsHandler): Lsp4jClient =
        Lsp4jClient(
            object : LspServerNotificationsHandler by handler {
                override fun logMessage(params: MessageParams) {
                    Log.of(project).add(params.message)
                    handler.logMessage(params)
                }
            },
        )

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

        /** The running clients of [project]. */
        fun running(project: Project): Collection<LspClient> =
            LspClientManager.getInstance(project).getClients(Provider::class.java)

        /** Restarts the server in [project], asking it for its file types again. */
        fun restart(project: Project) {
            Server.reset()
            reported.set(false)
            LspClientManager.getInstance(project)
                .stopAndRestartClientsIfNeeded(Provider::class.java)
        }
    }
}
