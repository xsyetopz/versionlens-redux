"""Small synchronous helpers for benchmark LSP clients."""

import json


def send(process, message):
    body = json.dumps(message).encode()
    process.stdin.write(f"Content-Length: {len(body)}\r\n\r\n".encode() + body)
    process.stdin.flush()


def read_message(process):
    length = None
    while line := process.stdout.readline():
        if line == b"\r\n":
            break
        if line.lower().startswith(b"content-length:"):
            length = int(line.split(b":", 1)[1])
    if length is None:
        raise RuntimeError("LSP closed stdout before responding")
    return json.loads(process.stdout.read(length))


def answer_configuration(process, settings):
    """Answers the server's first `workspace/configuration` request with
    `settings` as the `versionlens` section of every folder."""
    while True:
        message = read_message(process)
        if message.get("method") != "workspace/configuration":
            continue
        items = message["params"]["items"]
        result = [
            settings if item["section"] == "versionlens" else None for item in items
        ]
        send(process, {"jsonrpc": "2.0", "id": message["id"], "result": result})
        return


def receive(process, request_id, on_notification=None):
    while True:
        message = read_message(process)
        if "method" in message and on_notification is not None:
            on_notification(message)
        if message.get("id") != request_id:
            continue
        if "error" in message:
            raise RuntimeError(message["error"])
        return message["result"]
