package com.versionlens.jetbrains

import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.wm.ToolWindowManager
import org.eclipse.lsp4j.ExecuteCommandParams

/** Runs one server command; [file] commands act on the manifest in the editor. */
internal abstract class ServerAction(private val command: String, private val file: Boolean) :
    AnAction(), DumbAware {
    override fun getActionUpdateThread(): ActionUpdateThread = ActionUpdateThread.BGT

    override fun update(e: AnActionEvent) {
        val project = e.project
        val target = e.getData(CommonDataKeys.VIRTUAL_FILE)
        e.presentation.isEnabled =
            project != null &&
                Client.running(project).isNotEmpty() &&
                (!file || (target != null && Client.supports(project, target)))
    }

    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val client = Client.running(project).firstOrNull() ?: return
        val target = e.getData(CommonDataKeys.VIRTUAL_FILE)
        val arguments: List<Any> =
            if (file && target != null) listOf(client.descriptor.getFileUri(target))
            else emptyList()
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

/** Updates every dependency in the manifest in the editor. */
internal class UpdateAllAction : ServerAction("versionlens.updateAll", true)

/** Shows or hides the lenses and hints. */
internal class ToggleAction : ServerAction("versionlens.toggle", false)

/** Drops every cached registry answer and fetches again. */
internal class ClearCacheAction : ServerAction("versionlens.clearCache", false)

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
        val project = e.project ?: return
        ToolWindowManager.getInstance(project).getToolWindow("VersionLens")?.show()
    }
}
