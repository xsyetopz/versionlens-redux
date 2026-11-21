plugins {
    id("bedrock.kotlin")
    alias(libs.plugins.intellij.platform)
}

group = "com.versionlens"

version = "0.6.3"

val repositoryRoot = rootProject.layout.projectDirectory

// Release builds stage each target's server as <dir>/<rust-target>/versionlens-lsp[.exe] and
// pass -PversionlensBinariesDir=<dir>; the plugin starts bin/<rust-target>/ for the running
// machine.
val binaries = providers.gradleProperty("versionlensBinariesDir").map { rootProject.file(it) }

dependencies {
    intellijPlatform {
        intellijIdea("2026.1.4")
        bundledModule("com.intellij.modules.lsp")
        bundledModule("com.intellij.modules.ultimate")
        zipSigner()
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

tasks.prepareSandbox {
    val staged = binaries
    doFirst {
        staged.orNull?.let { dir ->
            check(dir.listFiles().orEmpty().isNotEmpty()) {
                "versionlensBinariesDir $dir has no servers"
            }
        }
    }
    from(staged) {
        into(pluginName.map { "$it/bin" })
        filePermissions { unix("rwxr-xr-x") }
    }
}

tasks.buildPlugin { archiveBaseName = "versionlens-redux-jetbrains-plugin" }

// Publishing a release signs and uploads the zip the release workflow built, given as
// -PversionlensReleasePlugin=<zip>, rather than rebuilding it.
providers.gradleProperty("versionlensReleasePlugin").orNull?.let { release ->
    tasks.signPlugin { archiveFile = file(release) }
}
