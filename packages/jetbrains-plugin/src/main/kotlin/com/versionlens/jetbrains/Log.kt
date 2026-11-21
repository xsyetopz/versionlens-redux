package com.versionlens.jetbrains

import com.intellij.execution.filters.TextConsoleBuilderFactory
import com.intellij.execution.ui.ConsoleView
import com.intellij.execution.ui.ConsoleViewContentType
import com.intellij.openapi.components.Service
import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.project.Project
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import java.time.LocalTime
import java.time.format.DateTimeFormatter

/** The server's log lines for one project, kept until the VersionLens tool window shows them. */
@Service(Service.Level.PROJECT)
internal class Log {
    private val lines = ArrayDeque<String>()
    private var console: ConsoleView? = null

    /** Appends one timestamped line. */
    @Synchronized
    fun add(message: String) {
        val line = "${LocalTime.now().format(TIME)} $message\n"
        lines.addLast(line)
        if (lines.size > LIMIT) lines.removeFirst()
        console?.print(line, ConsoleViewContentType.NORMAL_OUTPUT)
    }

    /** Shows every kept line in [view], then each new one. */
    @Synchronized
    fun attach(view: ConsoleView) {
        console = view
        lines.forEach { view.print(it, ConsoleViewContentType.NORMAL_OUTPUT) }
    }

    @Synchronized
    fun detach(view: ConsoleView) {
        if (console === view) console = null
    }

    companion object {
        private const val LIMIT = 2000
        private val TIME = DateTimeFormatter.ofPattern("HH:mm:ss")

        /** The log of [project]. */
        fun of(project: Project): Log = project.getService(Log::class.java)
    }
}

/** The VersionLens tool window: a console with the server's log. */
internal class LogWindow : ToolWindowFactory, DumbAware {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val view = TextConsoleBuilderFactory.getInstance().createBuilder(project).console
        val log = Log.of(project)
        Disposer.register(toolWindow.disposable, view)
        Disposer.register(view) { log.detach(view) }
        toolWindow.contentManager.addContent(
            ContentFactory.getInstance().createContent(view.component, "", false),
        )
        log.attach(view)
    }
}
