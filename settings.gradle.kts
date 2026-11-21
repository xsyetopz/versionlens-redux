// >>> jvm-bedrock
pluginManagement {
  includeBuild("gradle/bedrock")
  // Central first: the plugin portal mirrors it on demand and has missed Kotlin artifacts.
  repositories {
    mavenCentral()
    gradlePluginPortal()
    google()
  }
}

plugins { id("bedrock.settings") }

// <<< jvm-bedrock

// The IntelliJ Platform plugin resolves the IDE through its own repositories, which bedrock
// requires in settings rather than in projects.
apply(from = "packages/jetbrains-plugin/repositories.settings.gradle.kts")

rootProject.name = "versionlens-redux"

include("jetbrains-plugin")

project(":jetbrains-plugin").projectDir = file("packages/jetbrains-plugin")
