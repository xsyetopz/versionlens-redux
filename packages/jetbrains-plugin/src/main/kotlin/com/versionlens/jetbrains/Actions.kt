package com.versionlens.jetbrains

import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.openapi.wm.ToolWindowManager
import org.eclipse.lsp4j.ExecuteCommandParams

/**
 * Runs one server command. [file] commands act on the manifest in the editor, and [level] (latest,
 * major, minor, or patch) is the last argument of the update commands.
 */
internal abstract class ServerAction(
    private val command: String,
    private val file: Boolean,
    private val level: String? = null,
) : AnAction(), DumbAware {
    override fun getActionUpdateThread(): ActionUpdateThread = ActionUpdateThread.BGT

    override fun update(e: AnActionEvent) {
        val project = e.project
        val target = e.getData(CommonDataKeys.VIRTUAL_FILE)
        e.presentation.isEnabled =
            project != null &&
                Client.running(project).isNotEmpty() &&
                (!file || (target != null && Client.supports(project, target))) &&
                available(project, target)
    }

    /** Whether the server status of [target] allows this command; the default allows it. */
    protected open fun available(project: Project, target: VirtualFile?): Boolean = true

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val client = Client.running(project).firstOrNull() ?: return
        val target = e.getData(CommonDataKeys.VIRTUAL_FILE)
        val arguments = buildList {
            if (file && target != null) add(client.descriptor.getFileUri(target))
            level?.let(::add)
        }
        ApplicationManager.getApplication().executeOnPooledThread {
            try {
                client.sendRequestSync(TIMEOUT) {
                    it.workspaceService.executeCommand(ExecuteCommandParams(command, arguments))
                }
            } catch (error: Exception) {
                Log.of(project).add("$command failed: ${error.message}")
            }
        }
    }

    private companion object {
        const val TIMEOUT = 30_000
    }
}

/** Fetches the versions of the manifest in the editor again. */
internal class RefreshAction : ServerAction("versionlens.refresh", true)

/** Drops every cached registry answer and fetches again. */
internal class ClearCacheAction : ServerAction("versionlens.clearCache", false)

/** Updates every dependency in the manifest in the editor to the latest versions. */
internal class UpdateAllLatestAction : ServerAction("versionlens.updateAll", true, "latest")

/** Updates every dependency in the manifest in the editor within the major version. */
internal class UpdateAllMajorAction : ServerAction("versionlens.updateAll", true, "major")

/** Updates every dependency in the manifest in the editor within the minor version. */
internal class UpdateAllMinorAction : ServerAction("versionlens.updateAll", true, "minor")

/** Updates every dependency in the manifest in the editor to patch releases. */
internal class UpdateAllPatchAction : ServerAction("versionlens.updateAll", true, "patch")

/** Updates every dependency in the workspace to the latest versions. */
internal class UpdateWorkspaceLatestAction :
    ServerAction("versionlens.updateWorkspace", false, "latest")

/** Updates every dependency in the workspace within the major version. */
internal class UpdateWorkspaceMajorAction :
    ServerAction("versionlens.updateWorkspace", false, "major")

/** Updates every dependency in the workspace within the minor version. */
internal class UpdateWorkspaceMinorAction :
    ServerAction("versionlens.updateWorkspace", false, "minor")

/** Updates every dependency in the workspace to patch releases. */
internal class UpdateWorkspacePatchAction :
    ServerAction("versionlens.updateWorkspace", false, "patch")

/** Sorts the dependencies of the manifest in the editor, when the server reports it sortable. */
internal class SortAction : ServerAction("versionlens.sort", true) {
    override fun available(project: Project, target: VirtualFile?): Boolean =
        target != null && Client.statusOf(project, target)?.sortable == true
}

/** Runs the install command configured for the manifest in the editor. */
internal class InstallAction : ServerAction("versionlens.install", true) {
    override fun available(project: Project, target: VirtualFile?): Boolean =
        target != null && Client.statusOf(project, target)?.install != null
}

/** Restarts the server. */
internal class RestartAction : AnAction(), DumbAware {
    override fun getActionUpdateThread(): ActionUpdateThread = ActionUpdateThread.BGT

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabled = e.project != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        e.project?.let(Client::restart)
    }
}

/** Opens the VersionLens tool window with the server log. */
internal class ShowLogAction : AnAction(), DumbAware {
    override fun getActionUpdateThread(): ActionUpdateThread = ActionUpdateThread.BGT

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabled = e.project != null
    }

    override fun actionPerformed(e: AnActionEvent) {
        e.project?.let(::showLog)
    }
}

/** Opens the VersionLens tool window of [project]. */
internal fun showLog(project: Project) {
    ToolWindowManager.getInstance(project).getToolWindow("VersionLens")?.show()
}
