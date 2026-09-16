"""Measure release LSP code lenses for 300 local packages, without network traffic."""

import json
import pathlib
import statistics
import subprocess
import tempfile
import threading
import time

from lsp_protocol import receive, send


def main():
    binary = pathlib.Path("target/release/versionlens-lsp").resolve()
    rustc = subprocess.run(
        ["rustc", "-Vv"], check=True, capture_output=True, text=True
    ).stdout
    target = next(
        line.removeprefix("host: ")
        for line in rustc.splitlines()
        if line.startswith("host: ")
    )
    with tempfile.TemporaryDirectory(prefix="versionlens-bench-") as directory:
        root = pathlib.Path(directory)
        dependencies = {}
        for index in range(300):
            name = f"local-{index:03}"
            member = root / "packages" / name
            member.mkdir(parents=True)
            (member / "package.json").write_text(
                json.dumps({"name": name, "version": "1.0.0"})
            )
            dependencies[name] = "workspace:*"
        text = json.dumps(
            {
                "private": True,
                "workspaces": ["packages/*"],
                "dependencies": dependencies,
            }
        )
        manifest = root / "package.json"
        manifest.write_text(text)
        process = subprocess.Popen(
            [str(binary)], stdin=subprocess.PIPE, stdout=subprocess.PIPE
        )
        watchdog = threading.Timer(30, process.kill)
        watchdog.start()
        try:
            send(
                process,
                {
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {"rootUri": root.as_uri(), "capabilities": {}},
                },
            )
            receive(process, 1)
            send(process, {"jsonrpc": "2.0", "method": "initialized", "params": {}})
            send(
                process,
                {
                    "jsonrpc": "2.0",
                    "method": "textDocument/didOpen",
                    "params": {
                        "textDocument": {
                            "uri": manifest.as_uri(),
                            "languageId": "json",
                            "version": 1,
                            "text": text,
                        }
                    },
                },
            )
            durations = []
            for request_id in range(2, 28):
                start = time.perf_counter()
                send(
                    process,
                    {
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "method": "textDocument/codeLens",
                        "params": {"textDocument": {"uri": manifest.as_uri()}},
                    },
                )
                lenses = receive(process, request_id)
                durations.append((time.perf_counter() - start) * 1000)
                if len(lenses) < 300:
                    raise AssertionError(
                        f"Expected at least 300 lenses; got {len(lenses)}"
                    )
            send(
                process,
                {"jsonrpc": "2.0", "id": 28, "method": "shutdown", "params": None},
            )
            receive(process, 28)
            start = time.perf_counter()
            send(process, {"jsonrpc": "2.0", "method": "exit", "params": None})
            process.wait(timeout=2)
            assert process.stdin is not None
            process.stdin.close()
            shutdown_ms = (time.perf_counter() - start) * 1000
            warm = sorted(durations[1:])
            print(
                json.dumps(
                    {
                        "samples": len(warm),
                        "cold_ms": durations[0],
                        "warm_median_ms": statistics.median(warm),
                        "warm_p95_ms": warm[int(len(warm) * 0.95)],
                        "warm_standard_deviation_ms": statistics.pstdev(warm),
                        "shutdown_ms": shutdown_ms,
                        "workload": "300 local npm workspace packages, 26 code-lens requests",
                        "toolchain": rustc.splitlines()[0],
                        "target": target,
                        "errors": 0,
                        "allocation_measurement": "not collected",
                    },
                    indent=2,
                )
            )
        finally:
            watchdog.cancel()
            if process.poll() is None:
                process.kill()
                process.wait()


if __name__ == "__main__":
    main()
