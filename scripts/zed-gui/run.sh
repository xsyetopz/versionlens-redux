#!/bin/sh
# Drives the VersionLens extension in Zed under Xvfb against the live registries, the way a user
# would, and fails on the first step that does not reach the server or show its answer:
#   1. the lenses and hints of package.json, Cargo.toml, and compose.yaml, with the deprecated,
#      directory, missing directory, yanked, and build statuses; compose.yaml opens as the
#      dockerfile extension's Docker Compose language;
#   1b. lockfiles: Cargo.lock's locked version as itself, a Julia Manifest.toml's advisories, and a
#      yarn.lock, which Zed has no language for, through the README's Plain Text mapping;
#   2. the build lens from the code actions menu, its prompt, and the choice written back;
#   3. every VersionLens command from the code actions menu, each answered without an error;
#   4. a command from the command palette, through `editor: toggle code actions`;
#   5. a command from the editor toolbar's Code Actions button;
#   6. the server's RPC messages in Zed's language server logs view, VS Code's trace.server;
#   7. Add URL authentication: its prompt, the stored file it opens, the value typed and saved,
#      and the server rereading it; then Remove URL authentication through its prompt.
# A wrapper records the server's traffic; screenshots, the traffic, and Zed's log go to the output
# directory.
# Usage: run.sh <extension.wasm> <output directory>, from the repository root.
set -eu
wasm=$1 out=$2
here=$(dirname "$0")
data=/root/.local/share/zed
ext=$data/extensions/installed/versionlens-lsp
settings=/root/.config/zed/settings.json
stored=/root/.config/versionlens/authorization.json
mkdir -p "$out" /zed "$ext" /root/.config/zed /proj/lib /cfg
rm -rf "$out"/*.png "$out"/*.log /cfg/lsp-*.log /cfg/lsp.lock "$data"/logs/Zed.log "$stored"

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
printf '[[package]]\nname = "time"\nversion = "0.3.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n' >/proj/Cargo.lock
printf '[[deps.LibCURL_jll]]\ndeps = ["Artifacts"]\nuuid = "deac9b47-8bc7-5906-a0fe-35ac56dc84c0"\nversion = "8.6.0+0"\n' >/proj/Manifest.toml
printf 'lodash@^4.17.0:\n  version "4.17.20"\n' >/proj/yarn.lock
# Zed applies a rewritten settings file at once; lenses start above the code and later move into
# the code actions menu, where a script can choose them.
write_settings() {
	cat >"$settings" <<J
{
  "code_lens": "$1",
  "inlay_hints": { "enabled": true },
  "toolbar": { "code_actions": true },
  "auto_update": false,
  "auto_install_extensions": { "toml": true, "dockerfile": true },
  "file_types": { "Plain Text": ["yarn.lock"] },
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
# Only the first server, the project's, is recorded: the stored authentication file lies outside
# the project, and Zed starts a second server for it whose traffic would interleave.
mkdir /cfg/lsp.lock 2>/dev/null || exec /target/release/versionlens-lsp "$@"
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

# 1b. A lockfile's version is the one installed, so time 0.3.0 satisfies only itself, and OSV checks
# the version a manifest records.
open_at /proj/Cargo.lock:3:12
wait_for 90 shows 'satisfies 0.3.0' || fail "Cargo.lock's time 0.3.0 did not resolve to itself"
open_at /proj/Manifest.toml:4:12
wait_for 90 shows 'LibCURL_jll 8.6.0+0 has known vulnerabilities' ||
	fail "Manifest.toml's LibCURL_jll 8.6.0+0 showed no advisories"
open_at /proj/yarn.lock:2:12
wait_for 90 shows 'lodash 4.17.20 has known vulnerabilities' || fail "yarn.lock never reached the server"
sleep 3
shot lockfiles
open_at /proj/compose.yaml:3:20
sleep 2

# Opens the code actions menu (ctrl-., editor::ToggleCodeActions). Zed fills it from the actions it
# fetched when the cursor moved, so the menu may bring no new request.
open_menu() {
	xdotool key ctrl+period
	sleep 2
	[ "$(lsp actions /cfg/lsp-out.log)" -gt 0 ] || fail "the code actions menu listed no VersionLens command"
}

# Runs a command palette entry by its name.
palette() {
	xdotool key ctrl+shift+p
	sleep 1
	xdotool type --delay 20 "$1"
	sleep 1
	xdotool key Return
	sleep 2
}

# Zed shows prompts in the bottom right quarter above the status bar, away from the menu at the
# cursor; before_prompt clears the notifications there and keeps the quarter, and click_prompt finds
# the prompt as what changed in it and clicks its last button, at its bottom, since the buttons
# answer only clicks.
quarter=800x460+800+500
before_prompt() {
	palette "workspace: clear all notifications"
	import -window root -crop $quarter +repage /cfg/before.png
}
click_prompt() {
	shot "$1"
	box=$(convert "$out/$1.png" -crop $quarter +repage /cfg/before.png -compose difference \
		-composite -threshold 5% -trim -format '%w %h %[fx:page.x] %[fx:page.y]' info: 2>/dev/null) || true
	set -- $box
	[ $# -eq 4 ] && [ "$1" -gt 40 ] || return 1
	xdotool mousemove $((800 + $3 + $1 / 2)) $((500 + $4 + $2 - 28)) click 1
}
# How many times the server sent $1.
sent() { grep -o "$1" /cfg/lsp-out.log | wc -l; }
sent_more() { [ "$(sent "$1")" -gt "$2" ]; }

# 2. The build lens, chosen from the code actions menu, asks which build to use.
write_settings menu
sleep 2
before_prompt
open_menu
shot build-menu
at=$(lsp lens /cfg/lsp-out.log versionlens.build) || fail "the build lens was not in the menu"
xdotool key Prior
[ "$at" -eq 0 ] || xdotool key --repeat "$at" Down
xdotool key Return
wait_for 20 lsp ran /cfg/lsp-in.log versionlens.build || fail "choosing change build never reached the server"
wait_for 20 shows 'window/showMessageRequest' || fail "the server never asked for a build"
sleep 2
click_prompt build-prompt || fail "the build prompt never appeared"
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
# Succeeds once Zed has asked the server to run $1 more than $2 times.
ran_more() { [ "$(lsp runs /cfg/lsp-in.log "$1")" -gt "$2" ]; }
open_at /proj/package.json:3:5
step=Right
# Chooses versionlens.$1 from the menu that the command $2 opens, with $3 naming that way in a
# failure, and waits for it to reach the server.
choose() {
	runs=$(lsp runs /cfg/lsp-in.log "versionlens.$1")
	seen=$(lsp actions /cfg/lsp-out.log $package)
	xdotool key $step
	if [ $step = Right ]; then step=Left; else step=Right; fi
	wait_for 20 newer_menu "$seen" || fail "Zed never asked for the actions of package.json again"
	$2
	at=$(lsp pick /cfg/lsp-out.log $package "versionlens.$1") || fail "the menu did not offer versionlens.$1"
	xdotool key Next
	[ "$at" -eq 0 ] || xdotool key --repeat "$at" Up
	xdotool key Return
	wait_for 20 ran_more "versionlens.$1" "$runs" || fail "choosing versionlens.$1 from $3 never reached the server"
	sleep 2
}
for command in clearCache refresh showPrereleases hidePrereleases hide show sort install updateAll updateWorkspace; do
	choose $command open_menu "the code actions menu"
done
shot commands

# 4. The command palette has no extension commands of its own; its `editor: toggle code actions`
# opens the same menu.
palette_menu() {
	palette "editor: toggle code actions"
	shot palette-menu
}
choose refresh palette_menu "the command palette"

# 5. The editor toolbar's Code Actions button (the bolt, toolbar.code_actions) opens it too.
toolbar_menu() {
	xdotool mousemove ${ZED_GUI_BOLT:-1265 89} click 1
	sleep 2
	shot toolbar-menu
}
choose clearCache toolbar_menu "the toolbar's Code Actions button"

# 6. VS Code's versionlens.trace.server is Zed's language server logs view: pick the versionlens
# server, turn on RPC Messages, move the cursor in compose.yaml so Zed asks for its actions, and
# copy the view's text, which must hold that request and its answer.
open_at /proj/compose.yaml:3:20
sleep 2
palette "dev: open language server logs"
sleep 3
xdotool mousemove 785 89 click 1
sleep 1
xdotool mousemove 745 117 click 1
sleep 2
shot trace-server
xdotool mousemove 870 89 click 1
sleep 1
shot trace-menu
xdotool mousemove 875 163 click 1
sleep 2
xdotool mousemove 400 196 click 1
sleep 1
xdotool key Up
sleep 1
xdotool key Down
sleep 4
shot trace-rpc
xdotool mousemove 1000 600 click 1
sleep 1
xdotool key ctrl+a ctrl+c
sleep 1
xclip -o -selection clipboard >"$out/rpc-messages.txt" 2>/dev/null || true
grep -q '^// Send:' "$out/rpc-messages.txt" && grep -q '^// Receive' "$out/rpc-messages.txt" &&
	grep -q '"method":"textDocument/codeAction".*file:///proj/compose.yaml' "$out/rpc-messages.txt" ||
	fail "the RPC Messages log never showed the code action request for compose.yaml"
# Closing the view, its pane's only tab, gives step 7 the layout its prompt search expects, and
# step 7 picks its commands from package.json's menus.
xdotool key ctrl+w
sleep 1
open_at /proj/package.json:3:5
sleep 2

# 7. Add URL authentication asks how the registry authenticates, writes the stored file with a
# blank to fill in, and opens it there; the last button is Header value.
asked=$(sent window/showMessageRequest)
before_prompt
choose addAuthentication open_menu "the code actions menu"
wait_for 20 sent_more window/showMessageRequest "$asked" || fail "Add URL authentication never asked how"
sleep 2
click_prompt auth-prompt || fail "the authentication prompt never appeared"
wait_for 20 lsp answered /cfg/lsp-in.log "Header value" || fail "the authentication prompt never sent Header value"
wait_for 20 test -s "$stored" || fail "Add URL authentication never wrote $stored"
wait_for 20 shows 'window/showDocument' || fail "the server never opened $stored"
sleep 3
xdotool type --delay 20 "Token zed-gui"
xdotool key ctrl+s
sleep 2
shot auth-file
grep -q '"https://registry.npmjs.org/": "Token zed-gui"' "$stored" ||
	fail "the typed value never reached $stored: $(cat "$stored")"
open_at /proj/package.json:3:5
wait_for 30 shows 'url authentication: 1 URLs' || fail "the server never reread $stored"

# Remove URL authentication asks which URL to forget, with one button for the one stored.
asked=$(sent window/showMessageRequest)
before_prompt
choose removeAuthentication open_menu "the code actions menu"
wait_for 20 sent_more window/showMessageRequest "$asked" || fail "Remove URL authentication never asked which"
sleep 2
click_prompt forget-prompt || fail "the forget prompt never appeared"
wait_for 20 shows 'authentication removed for https://registry.npmjs.org/' || fail "the URL was never forgotten"
[ "$(tr -d ' \n' <"$stored")" = '{}' ] || fail "$stored still holds $(cat "$stored")"
shot auth-removed

cp "$data/logs/Zed.log" /cfg/lsp-*.log "$out/" 2>/dev/null || true
failed=$(lsp failed /cfg/lsp-in.log /cfg/lsp-out.log)
[ -z "$failed" ] || fail "the server answered commands with errors: $failed"
if grep -i 'versionlens' "$out/Zed.log" | grep -q 'ERROR'; then
	fail "Zed logged a VersionLens error; see $out/Zed.log"
fi
echo "zed-gui: statuses shown, build chosen from its prompt, every command ran from the code actions menu," \
	"one each from the palette and the toolbar, RPC messages traced," \
	"URL authentication added and removed"
