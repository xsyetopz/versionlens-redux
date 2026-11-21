package com.versionlens.jetbrains

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ServerTest {
    @Test
    fun globsMatchWholePathSegmentsCaseInsensitively() {
        val patterns = listOf("package.json", "requirements/*.txt", "*.csproj").map(Server::pattern)

        assertTrue(Server.matches(patterns, "/repo/package.json"))
        assertTrue(Server.matches(patterns, "/repo/requirements/dev.txt"))
        assertTrue(Server.matches(patterns, "C:/repo/App.CSPROJ"))
        assertFalse(Server.matches(patterns, "/repo/mypackage.json"))
        assertFalse(Server.matches(patterns, "/repo/requirements/sub/dev.txt"))
    }

    @Test
    fun releaseTriplesFollowOsArchAndLibc() {
        assertEquals("aarch64-apple-darwin", Server.triple("Mac OS X", "aarch64", false))
        assertEquals("x86_64-pc-windows-msvc", Server.triple("Windows 11", "amd64", false))
        assertEquals("x86_64-unknown-linux-musl", Server.triple("Linux", "amd64", true))
        assertEquals("armv7-unknown-linux-gnueabihf", Server.triple("Linux", "arm", false))
        assertNull(Server.triple("SunOS", "sparc", false))
    }
}
