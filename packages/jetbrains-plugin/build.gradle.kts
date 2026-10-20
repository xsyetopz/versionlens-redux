import org.gradle.api.tasks.bundling.Zip
import org.jetbrains.intellij.platform.gradle.tasks.SignPluginTask

plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.4.0"
    id("org.jetbrains.intellij.platform") version "2.18.1"
}

group = "com.versionlens"
version = "0.5.0"

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

dependencyLocking {
    lockAllConfigurations()
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

kotlin {
    jvmToolchain(21)
}


val repositoryRoot = layout.projectDirectory.dir("../..")

// One plugin bundles every release target from scripts/package/targets.json. The
// binaries are staged beforehand as <versionlensBinariesDir>/<rust-target>/versionlens-lsp[.exe],
// for example with scripts/package/stage-lsp.mjs from the build-lsp archives.
data class ReleaseTarget(val editor: String, val rust: String) {
    val executable = if (rust.contains("windows")) "versionlens-lsp.exe" else "versionlens-lsp"
}

val releaseTargets: List<ReleaseTarget> =
    @Suppress("UNCHECKED_CAST")
    (groovy.json.JsonSlurper().parse(repositoryRoot.file("scripts/package/targets.json").asFile)
        as Map<String, List<Map<String, String>>>)
        .getValue("targets")
        .map { ReleaseTarget(it.getValue("vscode"), it.getValue("rust")) }
val binariesDirectory =
    layout.projectDirectory.dir(
        providers.gradleProperty("versionlensBinariesDir").orElse("build/lsp-binaries").get(),
    )
// Local escape hatch: bundle only the staged targets instead of failing on missing ones.
val allowPartialTargets = providers.gradleProperty("versionlensAllowPartialTargets").isPresent

fun stagedBinary(target: ReleaseTarget) =
    binariesDirectory.dir(target.rust).file(target.executable).asFile

val verifyVersionLensBinaries =
    tasks.register("verifyVersionLensBinaries") {
        val directory = binariesDirectory
        inputs.property("allowPartialTargets", allowPartialTargets)
        doLast {
            val missing = releaseTargets.filterNot { stagedBinary(it).isFile }
            if (missing.isNotEmpty() && (!allowPartialTargets || missing.size == releaseTargets.size)) {
                throw GradleException(
                    "Missing language server binaries in ${directory.asFile}: " +
                        missing.joinToString { "${it.rust}/${it.executable}" } +
                        ". Stage them with scripts/package/stage-lsp.mjs or set -PversionlensBinariesDir.",
                )
            }
        }
    }

dependencies {
    testImplementation(kotlin("test"))
    intellijPlatform {
        intellijIdea("2026.1.4")
        bundledModule("com.intellij.modules.lsp")
        bundledModule("com.intellij.modules.ultimate")
        zipSigner()
    }
}

tasks.test {
    useJUnitPlatform()
}

intellijPlatform {
    buildSearchableOptions = false

    pluginConfiguration {
        id = "com.versionlens.jetbrains"
        name = "VersionLens Redux"
        description = "VersionLens Redux dependency hints, diagnostics, and code lenses through the shared VersionLens language server."
        changeNotes = "Added whole-workspace checking, persistent caches, runtime version checks, and functional language-server updates. Requires IDE build 261.26222.65 or newer."
        ideaVersion {
            sinceBuild = "261.26222.65"
            untilBuild = provider { null }
        }
        vendor {
            name = "VersionLens contributors"
            url = "https://github.com/xsyetopz/versionlens-redux"
        }
    }


    pluginVerification {
        freeArgs = listOf("-ignore-os-arch")
        ides {
            create("IU", "2026.1.4")
            recommended()
        }
    }

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
    doLast {
        val pluginIcon = layout.buildDirectory.file("resources/main/META-INF/pluginIcon.svg").get().asFile
        pluginIcon.parentFile.mkdirs()
        repositoryRoot.file("assets/versionlens/logo-redux.svg").asFile.copyTo(pluginIcon, overwrite = true)
    }
}

tasks.named<Zip>("buildPlugin").configure {
    dependsOn(verifyVersionLensBinaries)
    releaseTargets.forEach { target ->
        from(binariesDirectory.dir(target.rust)) {
            include(target.executable)
            into("bin/${target.editor}")
            filePermissions {
                unix("rwxr-xr-x")
            }
        }
    }
}

// Publishing signs and uploads an already released plugin archive instead of rebuilding it.
providers.gradleProperty("versionlensReleasePlugin").orNull?.let { released ->
    tasks.named<SignPluginTask>("signPlugin").configure {
        archiveFile.set(layout.projectDirectory.file(released))
    }
}
