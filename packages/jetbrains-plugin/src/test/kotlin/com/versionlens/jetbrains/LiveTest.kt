package com.versionlens.jetbrains

import com.intellij.codeInsight.intention.IntentionManager
import com.intellij.openapi.actionSystem.ActionManager
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.actionSystem.impl.SimpleDataContext
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.application.EDT
import com.intellij.openapi.application.runReadActionBlocking
import com.intellij.openapi.application.writeIntentReadAction
import com.intellij.openapi.editor.Editor
import com.intellij.openapi.fileEditor.FileEditorManager
import com.intellij.openapi.fileEditor.OpenFileDescriptor
import com.intellij.openapi.project.Project
import com.intellij.platform.lsp.api.LspClient
import com.intellij.platform.lsp.api.LspServerState
import com.intellij.psi.PsiFile
import com.intellij.testFramework.PlatformTestUtil
import com.intellij.testFramework.TestActionEvent
import com.intellij.testFramework.common.timeoutRunBlocking
import com.intellij.testFramework.fixtures.impl.CodeInsightTestFixtureImpl
import com.intellij.testFramework.junit5.TestApplication
import com.intellij.testFramework.junit5.fixture.fileEditorManagerFixture
import com.intellij.testFramework.junit5.fixture.moduleFixture
import com.intellij.testFramework.junit5.fixture.projectFixture
import com.intellij.testFramework.junit5.fixture.psiFileFixture
import com.intellij.testFramework.junit5.fixture.sourceRootFixture
import kotlin.test.Test
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.minutes
import kotlinx.coroutines.Dispatchers
import org.eclipse.lsp4j.HoverParams
import org.eclipse.lsp4j.InlayHintParams
import org.eclipse.lsp4j.Position
import org.eclipse.lsp4j.Range
import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable

/**
 * Drives the plugin inside a headless IntelliJ IDEA against the real server, registries, and OSV.
 * `just jetbrains-live` builds the server and runs it; other test runs skip it.
 */
@TestApplication
@EnabledIfEnvironmentVariable(named = "VERSIONLENS_LIVE", matches = "1")
class LiveTest {
    private val project = projectFixture(openAfterCreation = true)
    private val root = project.moduleFixture().sourceRootFixture()
    private val manifest = root.psiFileFixture("requirements.txt", "requests==2.31.0\nflask>=2.0\n")
    private val notes = root.psiFileFixture("notes.txt", "requests==2.31.0\n")
    private val editors = project.fileEditorManagerFixture()

    @Test
    fun drivesTheServerFromTheIde(): Unit =
        timeoutRunBlocking(3.minutes, context = Dispatchers.EDT) {
            writeIntentReadAction { drive() }
        }

    private fun drive() {
        val project = project.get()
        editors.get()
        val file = manifest.get()
        val editor = open(project, file)
        val client = running(project)
        until("the manifest is read") { logged(project, "requirements manifest, 2 dependencies") }
        until("hints for both dependencies") { hints(client, file).size == 2 }
        val infos =
            until("an OSV diagnostic in the editor") {
                highlights(file, editor)
                    .filter { it.contains("known vulnerabilities") }
                    .ifEmpty { null }
            }
        println("PASS diagnostics: $infos")
        editor.caretModel.moveToOffset(2)
        val fix =
            until("an update quick fix on the package name") {
                intentions(project, file, editor).firstOrNull { it.startsWith("Update requests") }
            }
        println("PASS quick fix: $fix")
        val hover =
            client.sendRequestSync(TIMEOUT) {
                it.textDocumentService.hover(HoverParams(id(client, file), Position(0, 2)))
            }
        assertTrue(
            hover?.contents?.right?.value.orEmpty().contains("requests"),
            "hover names the package",
        )
        println("PASS hover")
        notes.get().let { open(project, it) }
        assertTrue(!logged(project, "notes.txt"), "other files never reach the server")
        settings(project, client, file)
        actions(project, client, file, editor)
    }

    private fun settings(project: Project, client: LspClient, file: PsiFile) {
        Settings.get().inlayHints = false
        Client.reconfigure(project)
        until("hints off after the setting changes") { hints(client, file).isEmpty() }
        Settings.get().inlayHints = true
        Client.reconfigure(project)
        until("hints back after the setting resets") { hints(client, file).size == 2 }
        println("PASS settings apply without a restart")
    }

    private fun actions(project: Project, client: LspClient, file: PsiFile, editor: Editor) {
        run(project, file, "versionlens.toggle")
        until("toggle hides the hints") {
            logged(project, "VersionLens hidden") && hints(client, file).isEmpty()
        }
        run(project, file, "versionlens.toggle")
        until("toggle shows them again") {
            logged(project, "VersionLens shown") && hints(client, file).size == 2
        }
        run(project, file, "versionlens.clearCache")
        run(project, file, "versionlens.refresh")
        until("clearCache and refresh run") { logged(project, "command versionlens.refresh") }
        run(project, file, "versionlens.updateAll")
        until("updateAll edits the document") { !editor.document.text.contains("==2.31.0") }
        println("PASS updateAll: ${editor.document.text.trim().replace("\n", " | ")}")
        run(project, file, "versionlens.showOutput")
        val starts = Log.of(project).lines().count { "starting" in it }
        run(project, file, "versionlens.restart")
        until("restart brings the server back") {
            Log.of(project).lines().count { "starting" in it } > starts &&
                running(project) !== client
        }
        println("PASS actions")
    }

    private fun run(project: Project, file: PsiFile, id: String) {
        val action = ActionManager.getInstance().getAction(id) ?: error("$id is not registered")
        val context =
            SimpleDataContext.builder()
                .add(CommonDataKeys.PROJECT, project)
                .add(CommonDataKeys.VIRTUAL_FILE, file.virtualFile)
                .build()
        val event = TestActionEvent.createTestEvent(action, context)
        action.update(event)
        assertTrue(event.presentation.isEnabled, "$id is enabled")
        action.actionPerformed(event)
    }

    private fun open(project: Project, file: PsiFile): Editor =
        FileEditorManager.getInstance(project)
            .openTextEditor(OpenFileDescriptor(project, file.virtualFile), true)
            ?: error("no editor for ${file.name}")

    private fun running(project: Project): LspClient =
        until("the server starts") {
            Client.running(project).firstOrNull { it.state == LspServerState.Running }
        }

    private fun id(client: LspClient, file: PsiFile) =
        client.getDocumentIdentifier(file.virtualFile)

    private fun hints(client: LspClient, file: PsiFile): List<Any> =
        client
            .sendRequestSync(TIMEOUT) {
                it.textDocumentService.inlayHint(
                    InlayHintParams(id(client, file), Range(Position(0, 0), Position(100, 0))),
                )
            }
            .orEmpty()

    private fun highlights(file: PsiFile, editor: Editor): List<String> =
        CodeInsightTestFixtureImpl.instantiateAndRun(file, editor, IntArray(0), true).mapNotNull {
            it.description
        }

    // The platform asks the server for code actions only off the EDT, as the intention popup does.
    private fun intentions(project: Project, file: PsiFile, editor: Editor): List<String> =
        ApplicationManager.getApplication()
            .executeOnPooledThread<List<String>> {
                runReadActionBlocking {
                    IntentionManager.getInstance()
                        .availableIntentions
                        .filter { it.isAvailable(project, editor, file) }
                        .map { it.text }
                }
            }
            .get()

    private fun logged(project: Project, text: String) = Log.of(project).lines().any { text in it }

    private fun <T : Any> until(what: String, probe: () -> T?): T {
        val end = System.currentTimeMillis() + WAIT
        while (true) {
            probe()
                ?.takeIf { it != false }
                ?.let {
                    return it
                }
            check(System.currentTimeMillis() < end) {
                "timed out: $what\n${Log.of(project.get()).lines().joinToString("")}"
            }
            PlatformTestUtil.dispatchAllEventsInIdeEventQueue()
            Thread.sleep(100)
        }
    }

    private companion object {
        const val TIMEOUT = 10_000
        const val WAIT = 30_000L
    }
}
