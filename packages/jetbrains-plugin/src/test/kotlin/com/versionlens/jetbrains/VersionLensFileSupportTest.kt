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
}
