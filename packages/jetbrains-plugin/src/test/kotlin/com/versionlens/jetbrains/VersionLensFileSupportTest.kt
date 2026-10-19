package com.versionlens.jetbrains

import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import java.nio.file.Files

class VersionLensFileSupportTest {
    @Test
    fun recognizesSupportedNamesAndExtensionsCaseInsensitively() {
        assertTrue(VersionLensLspServerDescriptor.supportsFileName("package.json", "json"))
        assertTrue(VersionLensLspServerDescriptor.supportsFileName(".nvmrc", null))
        assertTrue(VersionLensLspServerDescriptor.supportsFileName(".node-version", null))
        assertTrue(VersionLensLspServerDescriptor.supportsFileName(".bun-version", null))
        assertTrue(VersionLensLspServerDescriptor.supportsFileName("rust-toolchain", null))
        assertTrue(VersionLensLspServerDescriptor.supportsFileName("rust-toolchain.toml", "toml"))
        assertTrue(VersionLensLspServerDescriptor.supportsFileName("Dockerfile.dev", "dev"))
        assertTrue(VersionLensLspServerDescriptor.supportsFileName("build.gradle.kts", "KTS"))
    }

    @Test
    fun rejectsUnrelatedFiles() {
        assertFalse(VersionLensLspServerDescriptor.supportsFileName("README.md", "md"))
        assertFalse(VersionLensLspServerDescriptor.supportsFileName("Dockerfilex", "txt"))
    }

    @Test
    fun resolvesThePluginRootFromAnInstalledLibraryJar() {
        val pluginRoot = Files.createTempDirectory("versionlens-plugin-root")
        val library = Files.createDirectories(pluginRoot.resolve("lib"))
            .resolve("versionlens-jetbrains-plugin.jar")
        Files.createFile(library)

        assertTrue(
            VersionLensLspServerDescriptor.pluginDirectoryForCodeSource(library) == pluginRoot,
        )
    }

    @Test
    fun picksTheReleaseTargetForThisMachine() {
        val target = VersionLensLspServerDescriptor::currentTarget
        assertTrue(target("Mac OS X", "aarch64", false) == "darwin-arm64")
        assertTrue(target("Windows 11", "amd64", false) == "win32-x64")
        assertTrue(target("Linux", "amd64", false) == "linux-x64")
        assertTrue(target("Linux", "amd64", true) == "alpine-x64")
        assertTrue(target("Linux", "aarch64", true) == "alpine-arm64")
        assertTrue(target("Linux", "arm", false) == "linux-armhf")
        assertTrue(target("Linux", "riscv64", false) == null)
        assertTrue(target("Mac OS X", "arm", false) == null)
    }

    @Test
    fun answersWorkspaceConfigurationWithAnObject() {
        val answer = VersionLensLspServerDescriptor::workspaceConfiguration
        assertTrue(answer("versionlens", "{}").toString() == "{}")
        assertTrue(answer("versionlens", "").toString() == "{}")
        assertTrue(answer("versionlens", "[1]").toString() == "{}")
        assertTrue(answer("versionlens", """{"a":1}""").toString() == """{"a":1}""")
        assertTrue(answer("files.exclude", "{}").toString() == "{}")
        assertTrue(answer("other", "{}") == null)
    }
}
