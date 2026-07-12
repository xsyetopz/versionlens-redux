plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.4.0"
    id("org.jetbrains.intellij.platform") version "2.17.0"
}

group = "com.versionlens"
version = "0.1.1"

repositories {
    mavenCentral()
    intellijPlatform {
        defaultRepositories()
    }
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

kotlin {
    jvmToolchain(21)
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
