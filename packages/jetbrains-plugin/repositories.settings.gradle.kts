// Adds the IntelliJ Platform repositories to settings, at the plugin version the catalog pins.
import org.jetbrains.intellij.platform.gradle.extensions.intellijPlatform

buildscript {
    val catalog = file("../../gradle/libs.versions.toml").readText()
    val version = Regex("""intellij-platform = "([^"]+)"""").find(catalog)!!.groupValues[1]
    repositories { gradlePluginPortal() }
    dependencies {
        classpath("org.jetbrains.intellij.platform:intellij-platform-gradle-plugin:$version")
    }
}

plugins.apply(
    org.jetbrains.intellij.platform.gradle.plugins.settings.IntelliJPlatformSettingsPlugin::class
        .java,
)

dependencyResolutionManagement { repositories { intellijPlatform { defaultRepositories() } } }
