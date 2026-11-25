package com.versionlens.jetbrains

import com.intellij.ide.util.ChooseElementsDialog
import com.intellij.openapi.actionSystem.ActionUpdateThread
import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.project.DumbAware
import com.intellij.openapi.project.Project
import com.intellij.openapi.project.ProjectManager
import com.intellij.openapi.ui.InputValidator
import com.intellij.openapi.ui.Messages
import java.util.Base64
import javax.swing.Icon

private const val TITLE = "VersionLens"

/** Accepts an http or https URL prefix; `=` would split the stored `prefix=header` line. */
private object UrlValidator : InputValidator {
    private val url = Regex("^https?://[^\\s=]+$")

    override fun checkInput(inputString: String?): Boolean =
        url.matches(inputString.orEmpty().trim())

    override fun canClose(inputString: String?): Boolean = checkInput(inputString)
}

/** Accepts any non-blank answer. */
private object FilledValidator : InputValidator {
    override fun checkInput(inputString: String?): Boolean = !inputString.isNullOrBlank()

    override fun canClose(inputString: String?): Boolean = checkInput(inputString)
}

/**
 * Changes the stored authorization lines off the UI thread, since PasswordSafe may block, then has
 * every open project's server pull the new section and shows [done] when it is not null.
 */
private fun store(project: Project, change: (String) -> String, done: String?) {
    ApplicationManager.getApplication().executeOnPooledThread {
        val settings = Settings.get()
        try {
            settings.authorization = change(settings.authorization)
        } catch (error: IllegalArgumentException) {
            Log.of(project).add("authorization not saved: ${error.message}")
            return@executeOnPooledThread
        }
        ProjectManager.getInstance().openProjects.forEach(Client::reconfigure)
        if (done != null) {
            ApplicationManager.getApplication().invokeLater {
                Messages.showInfoMessage(project, done, TITLE)
            }
        }
    }
}

/**
 * Asks how the registry authenticates and returns its Authorization header, or null when cancelled.
 */
private fun askHeader(project: Project): String? {
    val kinds = arrayOf("Basic (user and password)", "Bearer token", "Header value", "Cancel")
    val kind = Messages.showDialog(project, "How the registry authenticates", TITLE, kinds, 0, null)
    return when (kind) {
        0 -> {
            val user =
                Messages.showInputDialog(project, "User name", TITLE, null, "", FilledValidator)
            val password = user?.let {
                Messages.showPasswordDialog(project, "Password", TITLE, null)
            }
            password?.let {
                "Basic " + Base64.getEncoder().encodeToString("${user.trim()}:$it".toByteArray())
            }
        }
        1 ->
            Messages.showPasswordDialog(project, "Token", TITLE, null, FilledValidator)?.let {
                "Bearer ${it.trim()}"
            }
        2 ->
            Messages.showPasswordDialog(
                    project,
                    "Authorization header value",
                    TITLE,
                    null,
                    FilledValidator,
                )
                ?.trim()
        else -> null
    }
}

/** Base for the two authentication actions, which need only a project. */
internal abstract class AuthenticationAction : AnAction(), DumbAware {
    override fun getActionUpdateThread(): ActionUpdateThread = ActionUpdateThread.BGT

    override fun update(e: AnActionEvent) {
        e.presentation.isEnabled = e.project != null
    }
}

/**
 * VS Code's "Add url authentication": asks for a registry URL prefix and its credentials and stores
 * the Authorization header in PasswordSafe, with the lines the settings page edits.
 */
internal class AddAuthenticationAction : AuthenticationAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val url =
            Messages.showInputDialog(
                    project,
                    "Registry URL prefix the header is sent to",
                    TITLE,
                    null,
                    "https://",
                    UrlValidator,
                )
                ?.trim() ?: return
        val header = askHeader(project) ?: return
        store(
            project,
            { Section.withEntry(it, url, header) },
            "VersionLens: authentication saved for $url",
        )
    }
}

/** VS Code's "Remove url authentication": forgets the chosen stored URL prefixes. */
internal class RemoveAuthenticationAction : AuthenticationAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val app = ApplicationManager.getApplication()
        app.executeOnPooledThread {
            val urls =
                try {
                    Section.parseMap(Settings.get().authorization).keys.toList()
                } catch (error: IllegalArgumentException) {
                    Log.of(project).add("authorization unreadable: ${error.message}")
                    return@executeOnPooledThread
                }
            app.invokeLater { choose(project, urls) }
        }
    }

    private fun choose(project: Project, urls: List<String>) {
        if (urls.isEmpty()) {
            Messages.showInfoMessage(project, "VersionLens: no stored url authentication", TITLE)
            return
        }
        val dialog =
            object :
                ChooseElementsDialog<String>(
                    project,
                    urls,
                    "URLs to Forget",
                    "URLs to forget",
                    true,
                ) {
                override fun getItemText(item: String): String = item

                override fun getItemIcon(item: String): Icon? = null
            }
        val picked = dialog.showAndGetResult()
        if (picked.isNotEmpty()) store(project, { Section.without(it, picked) }, null)
    }
}
