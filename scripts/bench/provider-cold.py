"""Benchmark isolated cold and warm checks across built-in public provider routes."""

import argparse
import json
import os
import pathlib
import statistics
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass

sys.dont_write_bytecode = True

from lsp_protocol import receive, send

SAMPLES = 5
SAMPLE_SPACING_SECONDS = 30
WARM_LIMIT_MS = 10
STANDARD_DEPENDENCIES = 12
STRESS_DEPENDENCIES = 24
LIMITS = {
    "shared": (700, 900),
    "direct": (1_500, 2_500),
    "multi-hop": (2_500, 4_000),
}


@dataclass(frozen=True)
class Route:
    name: str
    ecosystem: str
    package: str
    requirement: str
    tier: str = "direct"
    case_kind: str | None = None
    registry_url: str | None = None
    prerelease: bool = True


DEFAULTS = {
    "Cargo": ("serde", "1.0.0"),
    "Composer": ("monolog/monolog", "2.0.0"),
    "Deno": ("@std/assert", "1.0.0"),
    "Dotnet": ("Newtonsoft.Json", "13.0.1"),
    "Docker": ("library/alpine", "3.18"),
    "Dub": ("vibe-d", "0.9.0"),
    "Go": ("github.com/stretchr/testify", "v1.8.0"),
    "Maven": ("org.slf4j:slf4j-api", "2.0.0"),
    "Npm": ("react", "18.0.0"),
    "Python": ("requests", "2.28.0"),
    "Pub": ("http", "1.0.0"),
    "Ruby": ("rake", "13.0.0"),
    "Hex": ("phoenix", "1.7.0"),
    "Opam": ("dune", "3.0.0"),
    "Hackage": ("aeson", "2.0.0.0"),
    "Julia": ("JSON", "1.0.0"),
    "Cran": ("ggplot2", "3.4.0"),
    "Conan": ("zlib", "1.2.13"),
    "Vcpkg": ("fmt", "9.1.0"),
    "Swift": ("apple/swift-log", "1.5.0"),
    "Zig": ("ziglibs/known-folders", "0.7.0"),
    "Nim": ("nimble", "0.14.0"),
    "LuaRocks": ("luasocket", "3.1.0"),
    "Cpan": ("Try::Tiny", "0.31"),
    "Haxelib": ("hxcpp", "4.3.0"),
    "Terraform": ("hashicorp/aws", "5.0.0"),
    "Helm": ("hello-world", "0.1.0"),
    "AnsibleGalaxy": ("community.general", "8.0.0"),
    "Bazel": ("rules_cc", "0.0.9"),
    "Nix": ("NixOS/nixpkgs", "nixos-23.11"),
    "Unity": ("com.unity.inputsystem", "1.7.0"),
    "CocoaPods": ("Alamofire", "5.8.0"),
    "Cpp": ("fmt", "9.1.0"),
    "GitHub": ("actions/checkout", "v4.0.0"),
}

NPM_PACKAGES = [
    ("react", "18.3.1"),
    ("typescript", "5.7.2"),
    ("lodash", "4.17.21"),
    ("express", "4.21.1"),
    ("axios", "1.7.9"),
    ("chalk", "5.4.1"),
    ("zod", "3.24.1"),
    ("vite", "6.0.5"),
    ("eslint", "9.17.0"),
    ("prettier", "3.4.2"),
    ("commander", "13.0.0"),
    ("semver", "7.6.3"),
    ("debug", "4.4.0"),
    ("dotenv", "16.4.7"),
    ("glob", "11.0.1"),
    ("minimist", "1.2.8"),
    ("rxjs", "7.8.1"),
    ("uuid", "11.0.3"),
    ("yargs", "17.7.2"),
    ("ajv", "8.17.1"),
    ("esbuild", "0.24.2"),
    ("fastify", "5.2.1"),
    ("moment", "2.30.1"),
    ("ws", "8.18.0"),
]


def default_routes():
    case_kinds = {"Cpp": "XmakeLua"}
    tiers = {
        "AnsibleGalaxy": "multi-hop",
        "Cargo": "multi-hop",
        "Cran": "shared",
        "Docker": "multi-hop",
        "GitHub": "multi-hop",
        "Go": "multi-hop",
        "Helm": "shared",
        "LuaRocks": "shared",
        "Nim": "shared",
        "Nix": "multi-hop",
        "Swift": "multi-hop",
        "Zig": "multi-hop",
    }
    routes = [
        Route(
            ecosystem.lower(),
            ecosystem,
            package,
            requirement,
            tier=tiers.get(ecosystem, "direct"),
            case_kind=case_kinds.get(ecosystem),
        )
        for ecosystem, (package, requirement) in DEFAULTS.items()
    ]
    routes.extend(
        [
            Route("docker-hub", "Docker", "library/alpine", "3.18", "multi-hop"),
            Route(
                "docker-mcr",
                "Docker",
                "mcr.microsoft.com/dotnet/runtime",
                "8.0",
                "multi-hop",
            ),
            Route(
                "docker-oci",
                "Docker",
                "ghcr.io/home-assistant/home-assistant",
                "2024.1",
                "multi-hop",
            ),
            Route("maven-central", "Maven", "org.slf4j:slf4j-api", "2.0.0"),
            Route(
                "maven-plugin-portal",
                "Maven",
                "com.diffplug.spotless:com.diffplug.spotless.gradle.plugin",
                "6.0.0",
                case_kind="GradleSettings",
            ),
            Route(
                "maven-clojars",
                "Maven",
                "metosin:malli",
                "0.14.0",
                case_kind="ClojureDepsEdn",
            ),
            Route("pypi-rss", "Python", "requests", "2.28.0"),
            Route(
                "pypi-json",
                "Python",
                "requests",
                "2.28.0",
                registry_url="https://pypi.org/pypi/{name}/json",
            ),
            Route(
                "pypi-simple-json",
                "Python",
                "requests",
                "2.28.0",
                registry_url="https://pypi.org/simple",
            ),
            Route(
                "stackage",
                "Hackage",
                "stackage-lts",
                "lts-22.43",
                case_kind="StackYaml",
            ),
            Route("helm-http", "Helm", "hello-world", "0.1.0", "shared"),
            Route("helm-oci", "Helm", "nginx", "15.0.0", "multi-hop"),
        ]
    )
    return routes


def arguments():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=pathlib.Path)
    parser.add_argument("--baseline-binary", type=pathlib.Path)
    parser.add_argument("--route", action="append")
    parser.add_argument("--stress", action="store_true")
    parser.add_argument("--samples", type=int, default=SAMPLES)
    parser.add_argument("--spacing", type=int, default=SAMPLE_SPACING_SECONDS)
    parser.add_argument("--list", action="store_true")
    return parser.parse_args()


def load_cases(repository):
    path = repository / "tests/fixtures/checking-coverage/manifest-cases.json"
    cases = json.loads(path.read_text())
    return {(case["ecosystem"], case["kind"]): case for case in cases}


def route_case(route, cases):
    candidates = [
        case
        for (ecosystem, _), case in cases.items()
        if ecosystem == route.ecosystem
        and (route.case_kind is None or case["kind"] == route.case_kind)
    ]
    if not candidates:
        raise RuntimeError(f"no manifest case for route {route.name}")
    case = candidates[0]
    if route.name == "stackage":
        return case, f"resolver: {route.requirement}\n"
    text = case["text"]
    if route.ecosystem in {"Swift", "Zig"}:
        text = text.replace("example/foo", route.package)
    elif ":" in case["package"] and ":" in route.package:
        old_group, old_name = case["package"].split(":", 1)
        new_group, new_name = route.package.split(":", 1)
        text = text.replace(old_name, new_name).replace(old_group, new_group)
    else:
        text = text.replace(case["package"], route.package)
    text = text.replace(case["requirement"], route.requirement)
    if route.name in {"helm", "helm-http"}:
        text = text.replace(
            "https://example.test/charts", "https://helm.github.io/examples"
        )
    if route.name == "helm-oci":
        text = text.replace(
            "https://example.test/charts", "oci://registry-1.docker.io/bitnamicharts"
        )
    return case, text


def workloads(route, case, text, dependency_count):
    if route.name == "npm":
        dependencies = ",".join(
            json.dumps(name) + ":" + json.dumps(version)
            for name, version in NPM_PACKAGES[:dependency_count]
        )
        return [
            (pathlib.Path(case["path"]), f'{{"dependencies":{{{dependencies}}}}}\n')
        ]
    return [
        (pathlib.Path(str(index)) / case["path"], text)
        for index in range(dependency_count)
    ]


def initialize(process, route, prerelease, root_uri):
    providers = {}
    if route.registry_url:
        providers["registryUrls"] = [
            {"ecosystem": route.ecosystem.lower(), "url": route.registry_url}
        ]
    send(
        process,
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "rootUri": root_uri,
                "capabilities": {},
                "initializationOptions": {
                    "showPrereleases": prerelease,
                    "showVulnerabilities": False,
                    "providers": providers,
                },
            },
        },
    )
    receive(process, 1)
    send(process, {"jsonrpc": "2.0", "method": "initialized", "params": {}})


def code_lenses(process, request_id, uri, diagnostics):
    start = time.perf_counter()
    send(
        process,
        {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "textDocument/codeLens",
            "params": {"textDocument": {"uri": uri}},
        },
    )

    def collect(message):
        if message.get("method") == "textDocument/publishDiagnostics":
            diagnostics.extend(message["params"].get("diagnostics", []))

    result = receive(process, request_id, collect)
    return (time.perf_counter() - start) * 1000, result


def normalized(value):
    if isinstance(value, dict):
        return {
            key: normalized(item)
            for key, item in value.items()
            if key not in {"documentUri", "range", "uri"}
        }
    if isinstance(value, list):
        return [normalized(item) for item in value]
    return value


def run_sample(binary, route, cases, prerelease, sample, dependency_count):
    case, text = route_case(route, cases)
    with tempfile.TemporaryDirectory(
        prefix=f"versionlens-{route.name}-{sample}-"
    ) as directory:
        root = pathlib.Path(directory)
        home = root / "home"
        home.mkdir()
        environment = os.environ.copy()
        environment["HOME"] = str(home)
        process = subprocess.Popen(
            [str(binary)],
            cwd=root,
            env=environment,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        )
        try:
            initialize(process, route, prerelease, root.as_uri())
            documents = []
            for relative, document_text in workloads(
                route, case, text, dependency_count
            ):
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(document_text)
                documents.append(path)
                send(
                    process,
                    {
                        "jsonrpc": "2.0",
                        "method": "textDocument/didOpen",
                        "params": {
                            "textDocument": {
                                "uri": path.as_uri(),
                                "languageId": case["language"],
                                "version": 1,
                                "text": document_text,
                            }
                        },
                    },
                )

            cold_start = time.perf_counter()
            cold = []
            diagnostics = []
            for index, path in enumerate(documents):
                _, lenses = code_lenses(
                    process, 100 + index, path.as_uri(), diagnostics
                )
                cold.append(lenses)
            cold_ms = (time.perf_counter() - cold_start) * 1000

            time.sleep(0.05)
            warm = []
            warm_requests = []
            for index, path in enumerate(documents):
                elapsed, lenses = code_lenses(
                    process, 200 + index, path.as_uri(), diagnostics
                )
                warm_requests.append(elapsed)
                warm.append(lenses)
            messages = [item.get("message", "") for item in diagnostics]
            indicators = (
                "429",
                "provider error",
                "rate limit",
                "request failed",
                "too many requests",
            )
            errors = [
                message
                for message in messages
                if any(indicator in message.lower() for indicator in indicators)
            ]
            if errors:
                raise RuntimeError(f"{route.name}: provider diagnostics: {errors}")
            if sum(map(len, cold)) < dependency_count:
                raise RuntimeError(
                    f"{route.name}: expected {dependency_count} dependency lenses; "
                    f"diagnostics: {messages}"
                )
            if normalized(cold) != normalized(warm):
                raise RuntimeError(
                    f"{route.name}: cold and warm outputs differ structurally"
                )

            send(
                process,
                {"jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": None},
            )
            receive(process, 2)
            send(process, {"jsonrpc": "2.0", "method": "exit", "params": None})
            process.wait(timeout=5)
            return {
                "sample": sample,
                "cold_ms": round(cold_ms, 3),
                "warm_ms": round(max(warm_requests), 3),
                "output": normalized(cold),
            }
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()


def summarize(route, mode, samples, baseline_samples):
    median_limit, sample_limit = LIMITS[route.tier]
    cold = [sample["cold_ms"] for sample in samples]
    warm = [sample["warm_ms"] for sample in samples]
    summary = {
        "route": route.name,
        "mode": mode,
        "tier": route.tier,
        "cold_median_ms": round(statistics.median(cold), 3),
        "cold_max_ms": max(cold),
        "warm_max_ms": max(warm),
        "within_advisory_target": statistics.median(cold) <= median_limit
        and max(cold) < sample_limit
        and max(warm) <= WARM_LIMIT_MS,
    }
    if not baseline_samples:
        return summary

    baseline_cold = [sample["cold_ms"] for sample in baseline_samples]
    baseline_warm = [sample["warm_ms"] for sample in baseline_samples]
    candidate_median = statistics.median(cold)
    baseline_median = statistics.median(baseline_cold)
    improvement = (baseline_median - candidate_median) * 100 / baseline_median
    wins = sum(
        candidate["cold_ms"] < baseline["cold_ms"]
        for candidate, baseline in zip(samples, baseline_samples, strict=True)
    )
    summary.update(
        {
            "baseline_cold_median_ms": round(baseline_median, 3),
            "baseline_cold_max_ms": max(baseline_cold),
            "baseline_warm_max_ms": max(baseline_warm),
            "median_improvement_percent": round(improvement, 3),
            "candidate_pair_wins": wins,
            "promotable": improvement >= 5
            and wins >= 4
            and candidate_median <= baseline_median * 1.05
            and max(cold) <= max(baseline_cold) * 1.10,
        }
    )
    return summary


def main():
    args = arguments()
    repository = pathlib.Path(__file__).resolve().parents[2]
    routes = default_routes()
    if args.list:
        print("\n".join(route.name for route in routes))
        return
    selected = set(args.route or [route.name for route in routes])
    routes = [route for route in routes if route.name in selected]
    missing = selected - {route.name for route in routes}
    if missing:
        raise SystemExit(f"unknown routes: {', '.join(sorted(missing))}")
    binary = (args.binary or repository / "target/release/versionlens-lsp").resolve()
    if not binary.is_file():
        raise SystemExit("missing release LSP; build versionlens-lsp --release first")
    baseline_binary = args.baseline_binary.resolve() if args.baseline_binary else None
    cases = load_cases(repository)
    count = STRESS_DEPENDENCIES if args.stress else STANDARD_DEPENDENCIES
    report = []
    for route in routes:
        modes = [False, True] if route.prerelease else [False]
        for prerelease in modes:
            samples = []
            baseline_samples = []
            for sample in range(1, args.samples + 1):
                if baseline_binary:
                    first, second = (
                        (baseline_binary, binary)
                        if sample % 2
                        else (binary, baseline_binary)
                    )
                    first_result = run_sample(
                        first, route, cases, prerelease, sample, count
                    )
                    time.sleep(args.spacing)
                    second_result = run_sample(
                        second, route, cases, prerelease, sample, count
                    )
                    result, baseline = (
                        (second_result, first_result)
                        if sample % 2
                        else (first_result, second_result)
                    )
                    if baseline["output"] != result["output"]:
                        raise RuntimeError(
                            f"{route.name}: candidate differs from exact baseline"
                        )
                    baseline_samples.append(baseline)
                else:
                    result = run_sample(binary, route, cases, prerelease, sample, count)
                samples.append(result)
                if sample != args.samples:
                    time.sleep(args.spacing)
            report.append(
                summarize(
                    route,
                    "prerelease" if prerelease else "stable",
                    samples,
                    baseline_samples,
                )
            )
    output = {
        "dependencies_per_suite": count,
        "samples": args.samples,
        "spacing_seconds": args.spacing,
        "routes": report,
        "all_within_advisory_targets": all(
            item["within_advisory_target"] for item in report
        ),
        "all_promotable": bool(baseline_binary)
        and all(item["promotable"] for item in report),
    }
    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
