#!/bin/sh
# Opens a package.json in Zed under Xvfb with the VersionLens extension installed and code lenses
# and inlay hints on, then opens the code actions menu and runs its first entry. A wrapper records
# the server's traffic. It fails when the server never starts, when the menu lists no VersionLens
# command, or when choosing one never reaches the server. Screenshots, the traffic, and Zed's log
# go to the output directory.
# Usage: run.sh <extension.wasm> <output directory>, from the repository root.
set -eu
wasm=$1 out=$2
data=/root/.local/share/zed
ext=$data/extensions/installed/versionlens-lsp
mkdir -p "$out" /zed "$ext" /root/.config/zed /proj /cfg
rm -f "$out"/*.png "$out"/*.log /cfg/lsp-*.log "$data"/logs/Zed.log

# The volume keeps Zed between runs; ZED_REFRESH=1 downloads the latest stable release again.
if [ ! -x /zed/zed.app/libexec/zed-editor ] || [ "${ZED_REFRESH:-}" = 1 ]; then
	rm -rf /zed/*
	curl -fsSL "https://cloud.zed.dev/releases/stable/latest/download?asset=zed&arch=$(uname -m)&os=linux" |
		tar xz -C /zed
fi
missing=$(ldd /zed/zed.app/libexec/zed-editor | sed -n 's/^[[:space:]]*\(.*\) => not found$/\1/p')
[ -z "$missing" ] || { echo "zed-gui: Zed needs $missing" >&2; exit 1; }

# Zed rebuilds its extension index from the installed directory at startup.
cp packages/zed-extension/extension.toml "$ext/"
cp "$wasm" "$ext/extension.wasm"
printf '{"dependencies":{"react":"^17.0.0","lodash":"^4.0.0","zod":"^3.0.0"}}\n' >/proj/package.json
cat >/root/.config/zed/settings.json <<'J'
{
  "code_lens": "on",
  "inlay_hints": { "enabled": true },
  "auto_update": false,
  "telemetry": { "diagnostics": false, "metrics": false },
  "restore_on_startup": "none",
  "session": { "trust_all_worktrees": true },
  "languages": { "JSON": { "language_servers": ["versionlens"] } },
  "lsp": { "versionlens": { "binary": { "path": "/cfg/versionlens-lsp" } } }
}
J
cat >/cfg/versionlens-lsp <<'S'
#!/bin/sh
tee -a /cfg/lsp-in.log | /target/release/versionlens-lsp "$@" | tee -a /cfg/lsp-out.log
S
chmod +x /cfg/versionlens-lsp

# The container runs as root, which Zed refuses without ZED_ALLOW_ROOT; it draws through lavapipe,
# which it warns about without ZED_ALLOW_EMULATED_GPU.
export DISPLAY=:99 ZED_ALLOW_ROOT=true ZED_ALLOW_EMULATED_GPU=1
Xvfb :99 -screen 0 1600x1000x24 >/dev/null 2>&1 &
/zed/zed.app/libexec/zed-editor /proj /proj/package.json >"$out/zed-stdout.log" 2>&1 &

# Waits up to $1 seconds for the command in the remaining arguments to succeed.
wait_for() {
	limit=$1
	shift
	n=0
	until "$@" >/dev/null 2>&1; do
		n=$((n + 1))
		[ "$n" -lt "$limit" ] || return 1
		sleep 1
	done
}

fail() {
	import -window root "$out/failed.png" || true
	cp "$data/logs/Zed.log" /cfg/lsp-*.log "$out/" 2>/dev/null || true
	echo "zed-gui: $1" >&2
	exit 1
}

wait_for 120 xdotool search --class zed || fail "the Zed window never appeared"
wait_for 60 pgrep -f /target/release/versionlens-lsp || fail "opening package.json never started versionlens-lsp"
# Lookups reach the live registries; the lenses settle once every one has answered.
wait_for 60 grep -q 'latest' /cfg/lsp-out.log || fail "no lens or hint reported a version"
sleep 3
xdotool search --class zed windowactivate --sync 2>/dev/null || true
import -window root "$out/editor.png"

# ctrl-. is editor::ToggleCodeActions in Zed's Linux keymap.
xdotool key ctrl+period
wait_for 20 grep -q 'source.versionlens' /cfg/lsp-out.log || fail "the code actions menu listed no VersionLens command"
sleep 2
import -window root "$out/actions.png"
# The per-dependency quick fixes come first and edit the file directly; the VersionLens commands
# follow them, so pagedown (editor::ContextMenuLast) selects one.
xdotool key Next Return
wait_for 20 grep -q 'workspace/executeCommand' /cfg/lsp-in.log || fail "choosing a code action never reached the server"
sleep 2
import -window root "$out/after.png"

cp "$data/logs/Zed.log" /cfg/lsp-*.log "$out/" 2>/dev/null || true
if grep -i 'versionlens' "$out/Zed.log" | grep -q 'ERROR'; then
	fail "Zed logged a VersionLens error; see $out/Zed.log"
fi
command=$(grep -o '"command":"versionlens\.[a-zA-Z.]*"' /cfg/lsp-in.log | tail -1)
echo "zed-gui: server started, code actions menu lists VersionLens commands, ran $command"
