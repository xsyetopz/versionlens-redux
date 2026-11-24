package com.versionlens.jetbrains

import com.google.gson.JsonParser
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class StatusTest {
    private fun parse(json: String) = Status.parse(JsonParser.parseString(json).asJsonObject)

    @Test
    fun parsesEveryField() {
        val status =
            parse(
                """{"uri":"file:///repo/package.json","enabled":true,"prereleases":true,
                "busy":false,"dependencies":12,"outdated":3,"failed":1,"vulnerable":2,
                "sortable":true,"install":"bun install"}""",
            )

        assertEquals(
            Status(
                "file:///repo/package.json",
                enabled = true,
                prereleases = true,
                busy = false,
                dependencies = 12,
                outdated = 3,
                failed = 1,
                vulnerable = 2,
                sortable = true,
                install = "bun install",
            ),
            status,
        )
    }

    @Test
    fun nullInstallAndMissingFieldsTakeDefaults() {
        val status = parse("""{"uri":"file:///a","install":null,"busy":true}""")!!

        assertNull(status.install)
        assertTrue(status.busy == true)
        assertEquals(0, status.failed)
        assertEquals(false, status.sortable)
    }

    @Test
    fun payloadWithoutUriIsRejected() {
        assertNull(parse("""{"enabled":true}"""))
    }
}
