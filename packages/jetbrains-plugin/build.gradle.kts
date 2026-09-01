import org.gradle.api.tasks.bundling.Zip

plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.4.0"
    id("org.jetbrains.intellij.platform") version "2.18.1"
}

group = "com.versionlens"
version = "0.4.0"

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
val lspRustTarget = providers.gradleProperty("versionlensRustTarget").orNull
val hostArchitecture = System.getProperty("os.arch").lowercase()
val hostRustTarget =
    when {
        System.getProperty("os.name").startsWith("Windows", ignoreCase = true) &&
            hostArchitecture in setOf("amd64", "x86_64") -> "x86_64-pc-windows-msvc"
        System.getProperty("os.name").startsWith("Windows", ignoreCase = true) &&
            hostArchitecture in setOf("arm64", "aarch64") -> "aarch64-pc-windows-msvc"
        System.getProperty("os.name").startsWith("Mac", ignoreCase = true) &&
            hostArchitecture in setOf("x86_64", "amd64") -> "x86_64-apple-darwin"
        System.getProperty("os.name").startsWith("Mac", ignoreCase = true) &&
            hostArchitecture in setOf("arm64", "aarch64") -> "aarch64-apple-darwin"
        System.getProperty("os.name").startsWith("Linux", ignoreCase = true) &&
            hostArchitecture in setOf("x86_64", "amd64") -> "x86_64-unknown-linux-gnu"
        System.getProperty("os.name").startsWith("Linux", ignoreCase = true) &&
            hostArchitecture in setOf("arm64", "aarch64") -> "aarch64-unknown-linux-gnu"
        else -> null
    }
val nativeTargets = mapOf(
    "x86_64-unknown-linux-gnu" to ("linux" to "x86_64"),
    "aarch64-unknown-linux-gnu" to ("linux" to "arm64"),
    "x86_64-apple-darwin" to ("mac" to "x86_64"),
    "aarch64-apple-darwin" to ("mac" to "arm64"),
    "x86_64-pc-windows-msvc" to ("windows" to "x86_64"),
    "aarch64-pc-windows-msvc" to ("windows" to "arm64"),
)
val selectedRustTarget = lspRustTarget ?: hostRustTarget
requireNotNull(selectedRustTarget) {
    "Unable to determine a supported native target from the host or versionlensRustTarget"
}
val selectedNativeTarget = requireNotNull(nativeTargets[selectedRustTarget]) {
    "Unsupported versionlensRustTarget=$selectedRustTarget; expected one of ${nativeTargets.keys}"
}
val (selectedOs, selectedArchitecture) = selectedNativeTarget
val variantVersion = "${project.version}-$selectedOs-$selectedArchitecture"
version = variantVersion
val lspExecutableName = if (selectedOs == "windows") "versionlens-lsp.exe" else "versionlens-lsp"
val lspOutputDirectory =
    if (lspRustTarget == null) {
        "target/release"
    } else {
        "target/$lspRustTarget/release"
    }
val lspBinary = repositoryRoot.file("$lspOutputDirectory/$lspExecutableName")
val buildVersionLensLsp =
    tasks.register<Exec>("buildVersionLensLsp") {
        workingDir(repositoryRoot)
        val cargoArguments =
            mutableListOf("cargo", "build", "-p", "versionlens-lsp", "--release", "--locked")
        if (lspRustTarget != null) {
            cargoArguments.addAll(listOf("--target", lspRustTarget))
        }
        commandLine(cargoArguments)
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
        version = variantVersion
        description = "VersionLens Redux dependency hints, diagnostics, and code lenses through the shared VersionLens language server."
        changeNotes = "Added target-specific native packages, safer language-server discovery, file-support tests, signing configuration, and Marketplace verification metadata for VersionLens Redux 0.4.0."
        ideaVersion {
            sinceBuild = "261"
            untilBuild = provider { null }
        }
        vendor {
            name = "VersionLens contributors"
            url = "https://github.com/xsyetopz/versionlens-redux"
        }
    }


    pluginVerification {
        ides {
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
        val descriptor = layout.buildDirectory.file("resources/main/META-INF/plugin.xml").get().asFile
        val marker = "<!-- native-variant-dependencies -->"
        val contents = descriptor.readText()
        require(contents.indexOf(marker) == contents.lastIndexOf(marker) && marker in contents) {
            "Expected exactly one native variant dependency marker in ${descriptor.path}"
        }
        descriptor.writeText(
            contents.replace(
                marker,
                "<depends>com.intellij.modules.os.$selectedOs</depends>\n  <depends>com.intellij.modules.arch.$selectedArchitecture</depends>",
            ),
        )
    }
}

tasks.named<Zip>("buildPlugin").configure {
    dependsOn(buildVersionLensLsp)
    from(lspBinary) {
        into("bin")
        filePermissions {
            unix("rwxr-xr-x")
        }
    }
}
