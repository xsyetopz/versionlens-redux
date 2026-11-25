package com.versionlens.jetbrains

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class TraceTest {
    private fun frame(body: String): ByteArray {
        val bytes = body.toByteArray(Charsets.UTF_8)
        return "Content-Length: ${bytes.size}\r\n\r\n".toByteArray(Charsets.US_ASCII) + bytes
    }

    @Test
    fun framerCutsMessagesSplitAnywhereAcrossWrites() {
        val bodies = listOf("""{"a":"zażółć"}""", "{}", """{"b":1}""")
        val stream = bodies.map(::frame).reduce(ByteArray::plus)
        val seen = mutableListOf<String>()
        val framer = Framer({ true }) { seen += it }

        for (b in stream) framer.feed(byteArrayOf(b), 0, 1)
        framer.feed(stream, 0, stream.size)

        assertEquals(bodies + bodies, seen)
    }

    @Test
    fun framerSkipsBodiesWhileOffAndPicksUpTheNextOne() {
        var on = false
        val seen = mutableListOf<String>()
        val framer = Framer({ on }) { seen += it }
        val first = frame("""{"skipped":true}""")
        val second = frame("""{"kept":true}""")

        // The switch comes after the first header, inside its body.
        val cut = first.size - 5
        framer.feed(first, 0, cut)
        on = true
        framer.feed(first, cut, first.size - cut)
        framer.feed(second, 0, second.size)

        assertEquals(listOf("""{"kept":true}"""), seen)
    }

    @Test
    fun tracesRequestsResponsesAndNotificationsAsVsCodeDoes() {
        val lines = mutableListOf<String>()
        val trace = Trace({ "messages" }, lines::add)

        trace.message(true, """{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}""", 100)
        trace.message(false, """{"jsonrpc":"2.0","id":1,"result":{}}""", 112)
        trace.message(
            false,
            """{"jsonrpc":"2.0","id":"c","method":"workspace/configuration"}""",
            200,
        )
        trace.message(true, """{"jsonrpc":"2.0","id":"c","result":[{}]}""", 203)
        trace.message(true, """{"jsonrpc":"2.0","method":"initialized","params":{}}""", 300)
        trace.message(
            false,
            """{"jsonrpc":"2.0","id":2,"error":{"code":-32601,"message":"no"}}""",
            301,
        )

        assertEquals(
            listOf(
                "[Trace] Sending request 'initialize - (1)'.",
                "[Trace] Received response 'initialize - (1)' in 12ms.",
                "[Trace] Received request 'workspace/configuration - (c)'.",
                "[Trace] Sending response 'workspace/configuration - (c)'. Processing request took 3ms",
                "[Trace] Sending notification 'initialized'.",
                "[Trace] Received response '? - (2)' in 0ms. Request failed: no (-32601).",
            ),
            lines,
        )
    }

    @Test
    fun verboseAddsTheParamsOrResult() {
        val lines = mutableListOf<String>()
        val trace = Trace({ "verbose" }, lines::add)

        trace.message(true, """{"jsonrpc":"2.0","method":"exit"}""")
        trace.message(
            false,
            """{"jsonrpc":"2.0","method":"window/logMessage","params":{"type":3}}""",
        )

        assertEquals("[Trace] Sending notification 'exit'.\nNo params.", lines[0])
        assertTrue(lines[1].endsWith("Params: {\n  \"type\": 3\n}"), lines[1])
    }

    @Test
    fun offTracesNothing() {
        val trace = Trace({ "off" }) { error("traced $it") }

        assertEquals(false, trace.enabled())
    }
}
