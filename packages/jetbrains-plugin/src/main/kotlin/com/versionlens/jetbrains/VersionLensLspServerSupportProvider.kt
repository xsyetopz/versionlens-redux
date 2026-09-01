package com.versionlens.jetbrains

import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.openapi.application.PathManager
import com.intellij.openapi.project.Project
import com.intellij.openapi.vfs.VirtualFile
import com.intellij.platform.lsp.api.LspIntegrationProvider
import com.intellij.platform.lsp.api.ProjectWideLspClientDescriptor
import java.io.IOException
import java.io.InputStream
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.util.Locale

internal class VersionLensLspServerSupportProvider : LspIntegrationProvider {
    override fun fileOpened(
        project: Project,
        file: VirtualFile,
        clientStarter: LspIntegrationProvider.LspClientStarter,
    ) {
        if (VersionLensLspServerDescriptor.supports(file)) {
            clientStarter.ensureClientStarted(VersionLensLspServerDescriptor(project))
        }
    }
}

internal class VersionLensLspServerDescriptor(
    private val currentProject: Project,
) : ProjectWideLspClientDescriptor(currentProject, SERVER_NAME) {
    override fun isSupportedFile(file: VirtualFile): Boolean = supports(file)

    override fun createCommandLine(): GeneralCommandLine = GeneralCommandLine(resolveServerPath())

    private fun resolveServerPath(): String {
        val propertyPath = System.getProperty("versionlens.lsp.path")
        if (!propertyPath.isNullOrBlank()) {
            return propertyPath
        }

        val environmentPath = System.getenv("VERSIONLENS_LSP")
        if (!environmentPath.isNullOrBlank()) {
            return environmentPath
        }

        installedPluginServerPath()?.let { return it }
        bundledServerPath()?.let { return it }

        val basePath = currentProject.basePath
        if (basePath != null) {
            val repoBinary = Path.of(basePath, "target", "debug", SERVER_BINARY)
            if (Files.isRegularFile(repoBinary)) {
                return repoBinary.toString()
            }
        }

        return SERVER_BINARY
    }

    private fun installedPluginServerPath(): String? {
        val codeSource = VersionLensLspServerSupportProvider::class.java.protectionDomain
            .codeSource?.location ?: return null
        return try {
            val location = Path.of(codeSource.toURI())
            val pluginDirectory = pluginDirectoryForCodeSource(location) ?: return null
            val binary = pluginDirectory.resolve("bin").resolve(SERVER_BINARY)
            if (!Files.isRegularFile(binary)) {
                null
            } else if (isWindows || Files.isExecutable(binary)) {
                binary.toString()
            } else {
                Files.newInputStream(binary).use(::materializeServer)
            }
        } catch (_: Exception) {
            null
        }
    }

    companion object {
        private const val SERVER_NAME = "VersionLens Redux"
        private val isWindows = System.getProperty("os.name").startsWith("Windows", ignoreCase = true)
        private val SERVER_BINARY =
            if (isWindows) {
                "versionlens-lsp.exe"
            } else {
                "versionlens-lsp"
            }

        internal fun pluginDirectoryForCodeSource(location: Path): Path? {
            val container = if (Files.isDirectory(location)) location else location.parent
            return if (container?.fileName?.toString() == "lib") container.parent else container
        }

        private fun bundledServerPath(): String? {
            val resource = VersionLensLspServerSupportProvider::class.java
                .getResourceAsStream("/bin/$SERVER_BINARY") ?: return null
            return resource.use(::materializeServer)
        }

        @Synchronized
        private fun materializeServer(resource: InputStream): String? {
            val directory = Path.of(PathManager.getSystemPath(), "versionlens-redux", "bin")
            val binary = directory.resolve(SERVER_BINARY)
            return try {
                Files.createDirectories(directory)
                val temporary = Files.createTempFile(directory, "$SERVER_BINARY.", ".tmp")
                try {
                    Files.copy(resource, temporary, StandardCopyOption.REPLACE_EXISTING)
                    if (!isWindows && !temporary.toFile().setExecutable(true)) {
                        return null
                    }
                    try {
                        Files.move(
                            temporary,
                            binary,
                            StandardCopyOption.REPLACE_EXISTING,
                            StandardCopyOption.ATOMIC_MOVE,
                        )
                    } catch (_: java.nio.file.AtomicMoveNotSupportedException) {
                        Files.move(temporary, binary, StandardCopyOption.REPLACE_EXISTING)
                    }
                } finally {
                    Files.deleteIfExists(temporary)
                }
                if (!Files.isRegularFile(binary) || (!isWindows && !Files.isExecutable(binary))) {
                    null
                } else {
                    binary.toString()
                }
            } catch (_: IOException) {
                null
            }
        }

        private val supportedFileNames = setOf(
            "WORKSPACE",
            "MODULE.bazel",
            "BUILD.bazel",
            "BUILD",
            "Dockerfile",
            "Gemfile",
            "Podfile",
            "cpanfile",
            "Pipfile",
            "requirements.txt",
            "paket.dependencies",
            "paket.references",
            "rebar.config",
            "stack.yaml",
            "pubspec.yaml",
            "pubspec_overrides.yaml",
            "deno.json",
            "deno.jsonc",
            "import_map.json",
            "composer.json",
            "package.json",
            "package.json5",
            "package.yaml",
            "pnpm-workspace.yaml",
            "Cargo.toml",
            "Package.swift",
            "go.mod",
            "pyproject.toml",
            "gleam.toml",
            "haxelib.json",
            "dub.json",
            "dub.sdl",
            "dune-project",
            "mix.exs",
            "flake.nix",
            "opam",
            "build.sbt",
            "xmake.lua",
            "build.zig.zon",
            "vcpkg.json",
            "conanfile.txt",
            "conanfile.py",
            "CMakeLists.txt",
        )
        private val supportedExtensions = setOf(
            "csproj",
            "fsproj",
            "vbproj",
            "props",
            "targets",
            "gradle",
            "kts",
            "pom",
            "xml",
            "json",
            "json5",
            "yaml",
            "yml",
            "toml",
            "lock",
            "gemspec",
            "rockspec",
            "nimble",
            "tf",
            "tfvars",
            "wrap",
            "cabal",
        )

        internal fun supportsFileName(name: String, extension: String?): Boolean {
            if (name in supportedFileNames) {
                return true
            }
            val lowerName = name.lowercase(Locale.ROOT)
            if (lowerName == "dockerfile" || lowerName.startsWith("dockerfile.")) {
                return true
            }
            return extension?.lowercase(Locale.ROOT) in supportedExtensions
        }

        fun supports(file: VirtualFile): Boolean {
            return supportsFileName(file.name, file.extension)
        }
    }
}
