#!/bin/sh
# Drives the VersionLens extension in Zed under Xvfb against the live registries, the way a user
# would, and fails on the first step that does not reach the server or show its answer:
#   1. the lenses and hints of package.json, Cargo.toml, and compose.yaml, with the deprecated,
#      directory, missing directory, yanked, and build statuses;
#   2. the build lens from the code actions menu, its prompt, and the choice written back;
#   3. every VersionLens command from the code actions menu, each answered without an error.
# A wrapper records the server's traffic; screenshots, the traffic, and Zed's log go to the output
# directory.
# Usage: run.sh <extension.wasm> <output directory>, from the repository root.
set -eu
wasm=$1 out=$2
here=$(dirname "$0")
data=/root/.local/share/zed
ext=$data/extensions/installed/versionlens-lsp
settings=/root/.config/zed/settings.json
mkdir -p "$out" /zed "$ext" /root/.config/zed /proj/lib /cfg
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
cat >/proj/package.json <<'J'
{
  "dependencies": {
    "react": "^17.0.0",
    "request": "2.88.2",
    "lib": "file:./lib",
    "gone": "file:./gone"
  }
}
J
printf '[package]\nname = "probe"\nversion = "0.1.0"\n\n[dependencies]\nrand = "=0.7.1"\n' >/proj/Cargo.toml
printf 'services:\n  web:\n    image: node:20-alpine\n' >/proj/compose.yaml
# Zed applies a rewritten settings file at once; lenses start above the code and later move into
# the code actions menu, where a script can choose them.
write_settings() {
	cat >"$settings" <<J
{
  "code_lens": "$1",
  "inlay_hints": { "enabled": true },
  "auto_update": false,
  "auto_install_extensions": { "toml": true },
  "telemetry": { "diagnostics": false, "metrics": false },
  "restore_on_startup": "none",
  "session": { "trust_all_worktrees": true },
  "languages": { "JSON": { "language_servers": ["versionlens"] } },
  "lsp": {
    "versionlens": {
      "binary": { "path": "/cfg/versionlens-lsp" },
      "settings": { "install": { "npm": "true" } }
    }
  }
}
J
}
write_settings on
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

lsp() { perl "$here/lsp.pl" "$@"; }
shows() { grep -q "$1" /cfg/lsp-out.log; }
shot() { import -window root "$out/$1.png"; }
# Opens a file at a line and column in the running Zed, the way `zed path:line:column` does.
open_at() { /zed/zed.app/bin/zed "$1" >>"$out/zed-stdout.log" 2>&1; }

wait_for 120 xdotool search --class zed || fail "the Zed window never appeared"
wait_for 60 pgrep -f /target/release/versionlens-lsp || fail "opening package.json never started versionlens-lsp"

# 1. Statuses, as the original showed them above each dependency.
wait_for 60 shows 'latest' || fail "no lens or hint reported a version"
wait_for 30 shows 'deprecated 2.88.2' || fail "request 2.88.2 never showed as deprecated"
wait_for 10 shows 'file://' || fail "the local directory dependency had no directory lens"
wait_for 10 shows 'not found' || fail "the missing directory dependency was not reported"
sleep 3
shot package
open_at /proj/Cargo.toml:6:1
wait_for 120 shows 'yanked 0.7.1' || fail "rand 0.7.1 never showed as yanked"
sleep 3
shot cargo
# Zed lists a line's lenses in the code actions menu only with the cursor inside their range, the tag.
open_at /proj/compose.yaml:3:20
wait_for 90 shows 'change build' || fail "node:20-alpine had no change build lens"
sleep 3
shot compose

# Opens the code actions menu (ctrl-., editor::ToggleCodeActions). Zed fills it from the actions it
# fetched when the cursor moved, so the menu may bring no new request.
open_menu() {
	xdotool key ctrl+period
	sleep 2
	[ "$(lsp actions /cfg/lsp-out.log)" -gt 0 ] || fail "the code actions menu listed no VersionLens command"
}

# 2. The build lens, chosen from the code actions menu, asks which build to use.
write_settings menu
sleep 2
# Zed shows prompts in the bottom right quarter, away from the menu at the cursor.
quarter=800x500+800+500
import -window root -crop $quarter +repage /cfg/before.png
open_menu
shot build-menu
at=$(lsp lens /cfg/lsp-out.log versionlens.build) || fail "the build lens was not in the menu"
xdotool key Prior
[ "$at" -eq 0 ] || xdotool key --repeat "$at" Down
xdotool key Return
wait_for 20 lsp ran /cfg/lsp-in.log versionlens.build || fail "choosing change build never reached the server"
wait_for 20 shows 'window/showMessageRequest' || fail "the server never asked for a build"
sleep 2
shot build-prompt
# The prompt's buttons answer only clicks: the prompt is what changed in that quarter, and its
# last button sits at the bottom of it.
box=$(convert "$out/build-prompt.png" -crop $quarter +repage /cfg/before.png -compose difference \
	-composite -threshold 5% -trim -format '%w %h %[fx:page.x] %[fx:page.y]' info: 2>/dev/null) || true
set -- $box
[ $# -eq 4 ] && [ "$1" -gt 40 ] || fail "the build prompt never appeared"
xdotool mousemove $((800 + $3 + $1 / 2)) $((500 + $4 + $2 - 20)) click 1
wait_for 20 lsp answered /cfg/lsp-in.log || fail "the build prompt never sent a choice"
wait_for 20 shows 'workspace/applyEdit' || fail "the chosen build was never written"
sleep 2
shot build-chosen

# 3. Every command, from the bottom of the menu, where they follow the per-dependency fixes. Zed
# fetches the focused document's actions again only when its cursor moves, so each pick moves the
# cursor to the other side of a character and waits for that document's new menu, which shows the last command's effect.
package=file:///proj/package.json
# Succeeds once package.json has more menus than $1; wait_for calls it again on each try.
newer_menu() { [ "$(lsp actions /cfg/lsp-out.log $package)" -gt "$1" ]; }
open_at /proj/package.json:3:5
step=Right
for command in clearCache refresh showPrereleases hidePrereleases hide show sort install updateAll updateWorkspace; do
	seen=$(lsp actions /cfg/lsp-out.log $package)
	xdotool key $step
	if [ $step = Right ]; then step=Left; else step=Right; fi
	wait_for 20 newer_menu "$seen" || fail "Zed never asked for the actions of package.json again"
	open_menu
	at=$(lsp pick /cfg/lsp-out.log $package "versionlens.$command") || fail "the menu did not offer versionlens.$command"
	xdotool key Next
	[ "$at" -eq 0 ] || xdotool key --repeat "$at" Up
	xdotool key Return
	wait_for 20 lsp ran /cfg/lsp-in.log "versionlens.$command" || fail "choosing versionlens.$command never reached the server"
	sleep 2
done
shot commands

cp "$data/logs/Zed.log" /cfg/lsp-*.log "$out/" 2>/dev/null || true
failed=$(lsp failed /cfg/lsp-in.log /cfg/lsp-out.log)
[ -z "$failed" ] || fail "the server answered commands with errors: $failed"
if grep -i 'versionlens' "$out/Zed.log" | grep -q 'ERROR'; then
	fail "Zed logged a VersionLens error; see $out/Zed.log"
fi
echo "zed-gui: statuses shown, build chosen from its prompt, every command ran from the code actions menu"
