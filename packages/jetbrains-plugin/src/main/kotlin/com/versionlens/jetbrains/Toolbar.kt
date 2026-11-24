package com.versionlens.jetbrains

import com.intellij.icons.AllIcons
import com.intellij.openapi.actionSystem.ActionGroup
import com.intellij.openapi.actionSystem.ActionManager
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.actionSystem.DefaultActionGroup
import com.intellij.openapi.actionSystem.toolbarLayout.ToolbarLayoutStrategy
import com.intellij.openapi.fileEditor.FileEditor
import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.IconLoader
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.ui.AnimatedIcon
import com.intellij.ui.EditorNotificationProvider
import com.intellij.util.ui.JBUI
import java.awt.Component
import java.awt.Graphics
import java.awt.Graphics2D
import java.util.function.Function
import javax.swing.Icon
import javax.swing.JComponent
import javax.swing.JPanel

/** The toolbar icons, ported from the VS Code extension's editor title icons. */
internal object Icons {
    private fun load(name: String): Icon = IconLoader.getIcon("/icons/$name.svg", Icons::class.java)

    val active = load("toggle_active")
    val inactive = load("toggle_inactive")
    private val mark = load("progress_mark")
    private val arrows = load("progress_arrows")

    // The VS Code icon spins its arrows around (7.5, 8) every 1.8 s; twelve frames of 150 ms match.
    val progress: Icon =
        AnimatedIcon(150, *Array<Icon>(12) { RotatedLayer(mark, arrows, it * 30.0) })
    val death = load("toggle_death")
    val tagActive = load("tag_active")
    val tagInactive = load("tag_inactive")
}

/** Paints [base], then [layer] turned by [degrees] around the VS Code icon's rotation centre. */
private class RotatedLayer(
    private val base: Icon,
    private val layer: Icon,
    private val degrees: Double,
) : Icon {
    override fun getIconWidth(): Int = base.iconWidth

    override fun getIconHeight(): Int = base.iconHeight

    override fun paintIcon(c: Component?, g: Graphics, x: Int, y: Int) {
        base.paintIcon(c, g, x, y)
        val turned = g.create() as Graphics2D
        try {
            turned.rotate(
                Math.toRadians(degrees),
                x + iconWidth * 7.5 / 16,
                y + iconHeight * 8.0 / 16,
            )
            layer.paintIcon(c, turned, x, y)
        } finally {
            turned.dispose()
        }
    }
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
            // Three buttons, the last a popup, stay on one line however narrow the editor is.
            toolbar.layoutStrategy = ToolbarLayoutStrategy.NOWRAP_STRATEGY
            JPanel(java.awt.BorderLayout()).apply {
                border = JBUI.Borders.empty()
                add(toolbar.component, java.awt.BorderLayout.WEST)
            }
        }
    }
}
