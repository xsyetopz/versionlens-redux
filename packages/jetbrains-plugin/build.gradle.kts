plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "2.4.0"
    id("org.jetbrains.intellij.platform") version "2.18.1"
}

group = "com.versionlens"
version = "0.1.2"

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


val lspExecutableName =
    if (System.getProperty("os.name").startsWith("Windows", ignoreCase = true)) {
        "versionlens-lsp.exe"
    } else {
        "versionlens-lsp"
    }
val repositoryRoot = layout.projectDirectory.dir("../..")
val lspBinary = repositoryRoot.file("target/release/$lspExecutableName")
val buildVersionLensLsp =
    tasks.register<Exec>("buildVersionLensLsp") {
        workingDir(repositoryRoot)
        commandLine("cargo", "build", "-p", "versionlens-lsp", "--release")
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
