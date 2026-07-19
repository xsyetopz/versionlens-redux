plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.4.0"
    id("org.jetbrains.intellij.platform") version "2.18.1"
}

group = "com.versionlens"
version = "0.2.0"

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


val lspExecutableName =
    if (System.getProperty("os.name").startsWith("Windows", ignoreCase = true)) {
        "versionlens-lsp.exe"
    } else {
        "versionlens-lsp"
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
require(lspRustTarget == null || lspRustTarget == hostRustTarget) {
    "versionlensRustTarget=$lspRustTarget does not match this native runner ($hostRustTarget)"
}
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

tasks.processResources {
    dependsOn(buildVersionLensLsp)
    from(lspBinary) {
        into("bin")
    }
}

dependencies {
    intellijPlatform {
        intellijIdea("2026.1.4")
        bundledModule("com.intellij.modules.lsp")
        bundledModule("com.intellij.modules.ultimate")
    }
}

intellijPlatform {
    pluginConfiguration {
        id = "com.versionlens.jetbrains"
        name = "VersionLens Redux"
        version = project.version.toString()
        description = "VersionLens Redux dependency hints, diagnostics, and code lenses through the shared VersionLens language server."
        vendor {
            name = "VersionLens contributors"
        }
    }
}
