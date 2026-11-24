package com.versionlens.jetbrains

import com.intellij.icons.AllIcons
import com.intellij.openapi.actionSystem.ActionGroup
import com.intellij.openapi.actionSystem.ActionManager
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.actionSystem.DefaultActionGroup
import com.intellij.openapi.fileEditor.FileEditor
import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.IconLoader
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.ui.EditorNotificationProvider
import com.intellij.util.ui.JBUI
import java.util.function.Function
import javax.swing.Icon
import javax.swing.JComponent
import javax.swing.JPanel

/** The toolbar icons, ported from the VS Code extension's editor title icons. */
internal object Icons {
    private fun load(name: String): Icon = IconLoader.getIcon("/icons/$name.svg", Icons::class.java)

    val active = load("toggle_active")
    val inactive = load("toggle_inactive")
    val progress = load("toggle_progress")
    val death = load("toggle_death")
    val tagActive = load("tag_active")
    val tagInactive = load("tag_inactive")
}

/** Shows or hides the versions; its icon shows the server state: busy, failed, shown, or hidden. */
internal class ToggleVersionsAction : ServerAction("versionlens.toggle", true) {
    override fun update(e: AnActionEvent) {
        super.update(e)
        val status = e.statusOrNull()
        val presentation = e.presentation
        when {
            status == null -> {
                presentation.icon = Icons.inactive
                presentation.text = "Show Versions"
            }
            status.failed > 0 -> {
                presentation.icon = Icons.death
                presentation.text = "${status.failed} lookups failed: show the VersionLens log"
            }
            status.busy -> {
                presentation.icon = Icons.progress
                presentation.text = "Fetching versions"
            }
            status.enabled -> {
                presentation.icon = Icons.active
                presentation.text = "Hide Versions"
            }
            else -> {
                presentation.icon = Icons.inactive
                presentation.text = "Show Versions"
            }
        }
    }

    override fun actionPerformed(e: AnActionEvent) {
        // The death icon opens the log instead, where the failures are.
        if ((e.statusOrNull()?.failed ?: 0) > 0) {
            e.project?.let { showLog(it) }
        } else {
            super.actionPerformed(e)
        }
    }
}

/** Switches between stable-only and prerelease versions for the manifest in the editor. */
internal class TogglePrereleasesAction : ServerAction("versionlens.togglePrereleases", true) {
    override fun update(e: AnActionEvent) {
        super.update(e)
        val on = e.statusOrNull()?.prereleases == true
        e.presentation.icon = if (on) Icons.tagActive else Icons.tagInactive
        e.presentation.text = if (on) "Hide Prereleases" else "Show Prereleases"
    }
}

/** The overflow popup of the toolbar. */
internal class MoreGroup : DefaultActionGroup(), DumbAware {
    init {
        isPopup = true
        templatePresentation.icon = AllIcons.Actions.More
        templatePresentation.text = "More VersionLens Actions"
    }

    override fun getActionUpdateThread(): ActionUpdateThread = ActionUpdateThread.BGT
}

private fun AnActionEvent.statusOrNull(): Status? {
    val project = project ?: return null
    val file = getData(CommonDataKeys.VIRTUAL_FILE) ?: return null
    return Client.statusOf(project, file)
}

/** Shows the VersionLens toolbar above each manifest the server reads. */
internal class ToolbarProvider : EditorNotificationProvider, DumbAware {
    override fun collectNotificationData(
        project: Project,
        file: VirtualFile,
    ): Function<in FileEditor, out JComponent?>? {
        if (!file.isInLocalFileSystem || !Client.supports(project, file)) return null
        return Function { editor ->
            val group = ActionManager.getInstance().getAction("versionlens.toolbar")
            val toolbar =
                ActionManager.getInstance()
                    .createActionToolbar("VersionLensEditorToolbar", group as ActionGroup, true)
            toolbar.targetComponent = editor.component
            JPanel(java.awt.BorderLayout()).apply {
                border = JBUI.Borders.empty()
                add(toolbar.component, java.awt.BorderLayout.WEST)
            }
        }
    }
}
