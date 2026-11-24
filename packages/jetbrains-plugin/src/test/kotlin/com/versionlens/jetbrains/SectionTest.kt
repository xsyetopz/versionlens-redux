package com.versionlens.jetbrains

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class SectionTest {
    @Test
    fun linesSplitOnTheFirstEqualsAndSkipComments() {
        val map =
            Section.parseMap(
                """
                # private registry
                npm = https://npm.example.com/{name}

                install=a=b && c
                npm=https://override/{name}
                """
                    .trimIndent(),
            )

        assertEquals(mapOf("npm" to "https://override/{name}", "install" to "a=b && c"), map)
    }

    @Test
    fun legacyJsonRegistriesStillParse() {
        assertEquals(
            mapOf("npm" to "https://npm.example.com/{name}"),
            Section.parseMap("""{"npm": "https://npm.example.com/{name}"}"""),
        )
    }

    @Test
    fun malformedLineNamesItsNumber() {
        val error = assertFailsWith<IllegalArgumentException> { Section.parseMap("a=b\nnope") }

        assertTrue(error.message!!.contains("line 2"))
        assertFailsWith<IllegalArgumentException> { Section.parseMap("=value") }
    }

    @Test
    fun buildMergesSecretsWithPlainValues() {
        val section =
            Section.build(
                Values(
                    prereleases = true,
                    timeoutMs = 5000,
                    registries = "npm=https://r/{name}",
                    install = "npm=npm install",
                    files = "*.deps=npm",
                ),
                githubToken = "ghp_x",
                authorization = "https://r/=Bearer abc",
            )

        assertEquals("ghp_x", section.get("githubToken").asString)
        assertTrue(section.get("prereleases").asBoolean)
        assertEquals(5000, section.get("timeoutMs").asInt)
        assertEquals(
            "Bearer abc",
            section.getAsJsonObject("authorization").get("https://r/").asString,
        )
        assertEquals("https://r/{name}", section.getAsJsonObject("registries").get("npm").asString)
        assertEquals("npm install", section.getAsJsonObject("install").get("npm").asString)
        assertEquals("npm", section.getAsJsonObject("files").get("*.deps").asString)
    }

    @Test
    fun blankTokenIsOmittedAndMapsAreEmptyObjects() {
        val section = Section.build(Values(), githubToken = " ", authorization = "")

        assertFalse(section.has("githubToken"))
        assertEquals(0, section.getAsJsonObject("authorization").size())
        assertEquals(0, section.getAsJsonObject("registries").size())
    }

    @Test
    fun listsSplitOnCommasAndNewlines() {
        val section =
            Section.build(
                Values(prereleaseTags = "rc, beta\n", fileTypes = "npm", updateSeverity = "hint"),
                githubToken = "",
                authorization = "",
            )

        assertEquals(
            listOf("rc", "beta"),
            section.getAsJsonArray("prereleaseTags").map { it.asString },
        )
        assertEquals(listOf("npm"), section.getAsJsonArray("fileTypes").map { it.asString })
        assertEquals(0, section.getAsJsonArray("workspaceExclude").size())
        assertEquals("hint", section.get("updateSeverity").asString)
    }

    @Test
    fun providerLinesBecomeTypedOverridesPerRegistry() {
        val providers =
            Section.providers("npm.cacheMinutes=5\nnpm.strictSsl=false\ngo.prereleaseTags=rc, beta")

        val npm = providers.getAsJsonObject("npm")
        assertEquals(5, npm.get("cacheMinutes").asInt)
        assertEquals(false, npm.get("strictSsl").asBoolean)
        assertEquals(2, providers.getAsJsonObject("go").getAsJsonArray("prereleaseTags").size())
    }

    @Test
    fun providerLinesRejectUnknownSettingsAndBadValues() {
        assertFailsWith<IllegalArgumentException> { Section.providers("npm.apiUrl=x") }
        assertFailsWith<IllegalArgumentException> { Section.providers("npm.strictSsl=maybe") }
        assertFailsWith<IllegalArgumentException> { Section.providers("cacheMinutes=5") }
    }
}
