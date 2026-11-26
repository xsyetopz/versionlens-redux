package com.versionlens.jetbrains

import com.google.gson.GsonBuilder
import com.google.gson.JsonElement
import com.google.gson.JsonParser
import com.intellij.execution.configurations.GeneralCommandLine
import com.intellij.execution.process.OSProcessHandler
import com.intellij.execution.process.ProcessEvent
import com.intellij.execution.process.ProcessListener
import com.intellij.execution.process.ProcessOutputTypes
import com.intellij.openapi.util.Key
import com.intellij.util.io.BaseDataReader
import com.intellij.util.io.BaseOutputReader
import java.io.FilterOutputStream
import java.io.OutputStream
import java.util.concurrent.ConcurrentHashMap

/** The trace levels of VS Code's `versionlens.trace.server`. */
internal val TRACE_LEVELS = listOf("off", "messages", "verbose")

/**
 * Starts the server as the IDE does and copies its LSP traffic to the VersionLens log while [level]
 * is `messages` or `verbose`. The IDE's LSP client offers no message hook, so the trace reads the
 * two pipes; it reads the level per message, so a change applies without a restart.
 */
internal fun tracedProcess(
    command: GeneralCommandLine,
    level: () -> String,
    log: (String) -> Unit,
): OSProcessHandler {
    val trace = Trace(level, log)
    val sent = Framer({ trace.enabled() }) { trace.message(true, it) }
    val received = Framer({ trace.enabled() }) { trace.message(false, it) }
    val handler =
        object : OSProcessHandler(command) {
            private val input by lazy { Tap(super.getProcessInput(), sent) }

            override fun getProcessInput(): OutputStream = input

            // The IDE's own server handler reads the output unsplit, polling like a quiet process.
            override fun readerOptions(): BaseOutputReader.Options =
                object : BaseOutputReader.Options() {
                    override fun policy(): BaseDataReader.SleepingPolicy =
                        forMostlySilentProcess().policy()

                    override fun splitToLines(): Boolean = false
                }
        }
    handler.addProcessListener(
        object : ProcessListener {
            override fun onTextAvailable(event: ProcessEvent, outputType: Key<*>) {
                if (outputType == ProcessOutputTypes.STDOUT) {
                    val bytes = event.text.toByteArray(command.charset)
                    received.feed(bytes, 0, bytes.size)
                }
            }
        },
    )
    return handler
}

/** Passes every byte written to the server on, and to [framer]. */
private class Tap(out: OutputStream, private val framer: Framer) : FilterOutputStream(out) {
    override fun write(b: Int) {
        out.write(b)
        framer.feed(byteArrayOf(b.toByte()), 0, 1)
    }

    override fun write(b: ByteArray, off: Int, len: Int) {
        out.write(b, off, len)
        framer.feed(b, off, len)
    }
}

/**
 * Cuts a `Content-Length` framed byte stream into message bodies for [emit]. Bodies arriving while
 * [keep] is false are skipped without being buffered.
 */
internal class Framer(private val keep: () -> Boolean, private val emit: (String) -> Unit) {
    private val head = StringBuilder()
    private var body: ByteArray? = null
    private var filled = 0
    private var left = -1

    /** Reads [len] bytes of [bytes] from [off]. */
    @Synchronized
    fun feed(bytes: ByteArray, off: Int, len: Int) {
        var i = off
        val end = off + len
        while (i < end) {
            if (left < 0) {
                head.append((bytes[i].toInt() and 0xff).toChar())
                i++
                if (head.endsWith("\r\n\r\n")) begin()
            } else {
                val n = minOf(left, end - i)
                body?.let { System.arraycopy(bytes, i, it, filled, n) }
                filled += n
                i += n
                left -= n
                if (left == 0) finish()
            }
        }
    }

    private fun begin() {
        val length = LENGTH.find(head)?.groupValues?.get(1)?.toIntOrNull() ?: 0
        head.setLength(0)
        body = if (keep()) ByteArray(length) else null
        filled = 0
        left = length
        if (left == 0) finish()
    }

    private fun finish() {
        body?.let { emit(String(it, Charsets.UTF_8)) }
        body = null
        left = -1
    }

    private companion object {
        val LENGTH = Regex("""Content-Length:\s*(\d+)""", RegexOption.IGNORE_CASE)
    }
}

/**
 * Formats each message as VS Code's language client traces it: a line naming the request, response,
 * or notification, and at `verbose` its params or result.
 */
internal class Trace(private val level: () -> String, private val log: (String) -> Unit) {
    private val started = ConcurrentHashMap<String, Pair<String, Long>>()

    /** Whether messages are traced now. */
    fun enabled(): Boolean = level() != "off"

    /** Traces one message body, [sending] it to the server or receiving it. */
    fun message(sending: Boolean, body: String, now: Long = System.currentTimeMillis()) {
        val verbose = level() == "verbose"
        val json =
            runCatching { JsonParser.parseString(body).asJsonObject }.getOrNull()
                ?: return log("[Trace] ${if (sending) "Sending" else "Received"} unreadable: $body")
        val way = if (sending) "Sending" else "Received"
        val method = json.get("method")?.asString
        val id = json.get("id")?.takeUnless { it.isJsonNull }?.let(::idText)
        val line: String
        val detail: Pair<String, JsonElement?>
        when {
            method != null && id != null -> {
                started["${if (sending) "out" else "in"}:$id"] = method to now
                line = "$way request '$method - ($id)'."
                detail = "Params" to json.get("params")
            }
            method != null -> {
                line = "$way notification '$method'."
                detail = "Params" to json.get("params")
            }
            else -> {
                // A response answers a request that went the other way.
                val (name, at) =
                    started.remove("${if (sending) "in" else "out"}:$id") ?: ("?" to now)
                val took = now - at
                val error = json.getAsJsonObject("error")
                line =
                    (if (sending)
                        "Sending response '$name - ($id)'. Processing request took ${took}ms"
                    else "Received response '$name - ($id)' in ${took}ms.") +
                        (error?.let {
                            " Request failed: ${it.get("message")?.asString} (${it.get("code")})."
                        } ?: "")
                detail =
                    if (error != null) "Error data" to error.get("data")
                    else "Result" to json.get("result")
            }
        }
        val (label, value) = detail
        log(
            "[Trace] $line" +
                if (!verbose) ""
                else if (value == null || value.isJsonNull)
                    "\n${if (label == "Result") "No result returned." else "No ${label.lowercase()}."}"
                else "\n$label: ${PRETTY.toJson(value)}",
        )
    }

    private fun idText(id: JsonElement): String =
        if (id.isJsonPrimitive && id.asJsonPrimitive.isString) id.asString else id.toString()

    private companion object {
        val PRETTY = GsonBuilder().setPrettyPrinting().serializeNulls().create()
    }
}
