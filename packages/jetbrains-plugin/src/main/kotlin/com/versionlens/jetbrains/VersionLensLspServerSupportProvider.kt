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
            ".bun-version",
            ".node-version",
            ".nvmrc",
            ".yarnrc.yaml",
            ".yarnrc.yml",
            "CMakeLists.txt",
            "Cargo.toml",
            "Cartfile",
            "Cartfile.private",
            "Chart.yaml",
            "DESCRIPTION",
            "Dockerfile",
            "Gemfile",
            "Kustomization",
            "MODULE.bazel",
            "Manifest.toml",
            "Package.swift",
            "Pipfile",
            "Podfile",
            "Project.toml",
            "WORKSPACE",
            "WORKSPACE.bazel",
            "build.gradle",
            "build.gradle.kts",
            "build.sbt",
            "build.zig.zon",
            "cabal.project",
            "compose.yaml",
            "compose.yml",
            "composer.json",
            "conanfile.py",
            "conanfile.txt",
            "cpanfile",
            "deno.json",
            "deno.jsonc",
            "deps.edn",
            "docker-compose.yaml",
            "docker-compose.yml",
            "dockerfile",
            "dotnet-tools.json",
            "dub.json",
            "dub.sdl",
            "dub.selections.json",
            "dune-project",
            "flake.nix",
            "gleam.toml",
            "go.mod",
            "go.work",
            "haxelib.json",
            "import_map.json",
            "jsr.json",
            "jsr.jsonc",
            "kustomization.yaml",
            "kustomization.yml",
            "libs.versions.toml",
            "mix.exs",
            "opam",
            "package.json",
            "package.json5",
            "package.yaml",
            "package.yml",
            "packages.config",
            "paket.dependencies",
            "paket.references",
            "pnpm-workspace.yaml",
            "pnpm-workspace.yml",
            "pom.xml",
            "project.clj",
            "project.json",
            "pubspec.yaml",
            "pubspec.yml",
            "pubspec_overrides.yaml",
            "pyproject.toml",
            "rebar.config",
            "renv.lock",
            "requirements.txt",
            "requirements.yaml",
            "requirements.yml",
            "rust-toolchain",
            "rust-toolchain.toml",
            "settings.gradle",
            "settings.gradle.kts",
            "stack.yaml",
            "stack.yml",
            "vcpkg.json",
            "xmake.lua",
        )
        private val supportedExtensions = setOf(
            "cabal",
            "cmake",
            "csproj",
            "dockerfile",
            "fsproj",
            "gemspec",
            "json",
            "nimble",
            "opam",
            "props",
            "rockspec",
            "targets",
            "tf",
            "tofu",
            "toml",
            "vbproj",
            "wrap",
            "xml",
            "yaml",
            "yml",
        )

        private val supportedPathSuffixes = setOf(
            ".cargo/config",
            ".cargo/config.toml",
            "Packages/manifest.json",
            "gradle/libs.versions.toml",
        )
        private val supportedNamePatterns = listOf<Pair<String, String>>(
            "constraints" to "txt",
            "requirements" to "txt",
        )

        internal fun supportsFileName(name: String, extension: String?, path: String? = null): Boolean {
            if (name in supportedFileNames) {
                return true
            }
            val lowerName = name.lowercase(Locale.ROOT)
            if (lowerName == "dockerfile" || lowerName.startsWith("dockerfile.")) {
                return true
            }
            val normalized = path?.replace('\\', '/')
            if (normalized != null &&
                supportedPathSuffixes.any { normalized == it || normalized.endsWith("/$it") }
            ) {
                return true
            }
            val lowerExtension = extension?.lowercase(Locale.ROOT)
            if (supportedNamePatterns.any { (contains, patternExtension) ->
                    lowerExtension == patternExtension && lowerName.contains(contains)
                }
            ) {
                return true
            }
            return lowerExtension in supportedExtensions
        }

        fun supports(file: VirtualFile): Boolean {
            return supportsFileName(file.name, file.extension, file.path)
        }
    }
}
