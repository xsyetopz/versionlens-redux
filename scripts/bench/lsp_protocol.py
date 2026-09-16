"""Small synchronous helpers for benchmark LSP clients."""

import json


def send(process, message):
    body = json.dumps(message).encode()
    process.stdin.write(f"Content-Length: {len(body)}\r\n\r\n".encode() + body)
    process.stdin.flush()


def receive(process, request_id, on_notification=None):
    while True:
        length = None
        while line := process.stdout.readline():
            if line == b"\r\n":
                break
            if line.lower().startswith(b"content-length:"):
                length = int(line.split(b":", 1)[1])
        if length is None:
            raise RuntimeError("LSP closed stdout before responding")
        message = json.loads(process.stdout.read(length))
        if "method" in message and on_notification is not None:
            on_notification(message)
        if message.get("id") != request_id:
            continue
        if "error" in message:
            raise RuntimeError(message["error"])
        return message["result"]
