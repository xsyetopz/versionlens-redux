import org.jetbrains.intellij.platform.gradle.TestFrameworkType

plugins {
    id("bedrock.kotlin")
    alias(libs.plugins.intellij.platform)
}

group = "com.versionlens"

version = "0.7.0"

val repositoryRoot = rootProject.layout.projectDirectory

// Release builds stage each target's server as <dir>/<rust-target>/versionlens-lsp[.exe] and
// pass -PversionlensBinariesDir=<dir>; the plugin starts bin/<rust-target>/ for the running
// machine.
val binaries = providers.gradleProperty("versionlensBinariesDir").map { rootProject.file(it) }

dependencies {
    // The IntelliJ test framework loads JUnit 4 classes at runtime; the tests use Jupiter.
    testRuntimeOnly(libs.junit4)
    intellijPlatform {
        intellijIdea("2026.1.4")
        bundledModule("com.intellij.modules.lsp")
        bundledModule("com.intellij.modules.ultimate")
        zipSigner()
        testFramework(TestFrameworkType.Platform)
        testFramework(TestFrameworkType.JUnit5)
    }
}

intellijPlatform {
    buildSearchableOptions = false
    pluginConfiguration {
        ideaVersion {
            sinceBuild = "261.26222.65"
            untilBuild = provider { null }
        }
    }
    pluginVerification { ides { create("IU", "2026.1.4") } }
    signing {
        certificateChain = providers.environmentVariable("JB_CERTIFICATE_CHAIN")
        privateKey = providers.environmentVariable("JB_PRIVATE_KEY")
        password = providers.environmentVariable("JB_PRIVATE_KEY_PASSWORD")
    }
    publishing {
        token = providers.environmentVariable("JETBRAINS_MARKETPLACE_TOKEN")
        channels = listOf("default")
    }
}

tasks.processResources {
    from(repositoryRoot.file("assets/versionlens/logo-redux.svg")) {
        into("META-INF")
        rename { "pluginIcon.svg" }
    }
}

// Without the property the plugin ships no server and finds one through VERSIONLENS_LSP, its
// settings, or PATH, as local GUI runs do.
tasks.prepareSandbox {
    binaries.orNull?.let { dir ->
        doFirst {
            check(dir.listFiles().orEmpty().isNotEmpty()) {
                "versionlensBinariesDir $dir has no servers"
            }
        }
        from(dir) {
            into(pluginName.map { "$it/bin" })
            filePermissions { unix("rwxr-xr-x") }
        }
    }
}

tasks.buildPlugin { archiveBaseName = "versionlens-redux-jetbrains-plugin" }

// Publishing a release signs and uploads the zip the release workflow built, given as
// -PversionlensReleasePlugin=<zip>, rather than rebuilding it.
providers.gradleProperty("versionlensReleasePlugin").orNull?.let { release ->
    tasks.signPlugin { archiveFile = file(release) }
}

// `just jetbrains-live` passes -PversionlensLiveServer=<versionlens-lsp> to run LiveTest against
// that server, the live registries, and OSV; other test runs skip LiveTest.
val liveServer = providers.gradleProperty("versionlensLiveServer")

tasks.test {
    inputs.property("versionlensLiveServer", liveServer.orElse(""))
    liveServer.orNull?.let {
        environment("VERSIONLENS_LSP", it)
        environment("VERSIONLENS_LIVE", "1")
        testLogging.showStandardStreams = true
    }
}

// Qodana's bundled Code Provenance plugin records edits after the test project is disposed, which
// the test framework reports as an uncaught exception; the plugin plays no part in VersionLens.
tasks.prepareTestSandbox { disabledPlugins.add("com.intellij.code.provenance") }
