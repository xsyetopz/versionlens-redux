#!/bin/sh
# Opens a package.json in IntelliJ IDEA under Xvfb with the VersionLens plugin installed, while a
# proxy that accepts and never answers keeps every lookup pending. It fails when the IDE blames
# the plugin for an error or a freeze, when the server never starts, or when the editor toolbar
# stays still while lookups are pending. Then it runs Add URL Authentication through Find Action and
# its dialogs, and Remove URL Authentication through its picker, and checks the server received
# the header and then lost it. Screenshots, the server's input, and the IDE log go to the output
# directory.
# Usage: run.sh <plugin zip> <output directory>, from the repository root; the IDE is the release
# the plugin builds against.
set -eu
zip=$1 out=$2
version=$(sed -n 's/.*intellijIdea("\(.*\)").*/\1/p' packages/jetbrains-plugin/build.gradle.kts)
[ -n "$version" ] || { echo "jetbrains-gui: no intellijIdea version in build.gradle.kts" >&2; exit 1; }
mkdir -p "$out" /idea /cfg/config /cfg/system /cfg/plugins /proj/.idea
rm -f "$out"/*.png "$out"/*.log /cfg/lsp-in.log

# The volume keeps the IDE between runs; a new pinned release replaces it.
if ! grep -qs "\"version\": \"$version\"" /idea/product-info.json; then
	rm -rf /idea/*
	arch=$([ "$(uname -m)" = aarch64 ] && echo -aarch64 || true)
	curl -fsSL "https://download.jetbrains.com/idea/idea-$version$arch.tar.gz" |
		tar xz -C /idea --strip-components=1
fi
unzip -qo "$zip" -d /cfg/plugins
# The container has no native keychain (libsecret over D-Bus), which PasswordSafe reports as an
# error blamed on the first plugin to read a password; the KeePass file store is a choice users have
# under Settings | Passwords too.
mkdir -p /cfg/config/options
# The trace copies the LSP traffic to the VersionLens tool window, shown at the end.
cat >/cfg/config/options/versionlens.xml <<'X'
<application>
  <component name="VersionLens">
    <option name="traceServer" value="messages" />
  </component>
</application>
X
cat >/cfg/config/options/security.xml <<'X'
<application>
  <component name="PasswordSafe">
    <option name="PROVIDER" value="KEEPASS" />
  </component>
</application>
X

printf '{"dependencies":{"react":"^17.0.0","lodash":"^4.0.0","zod":"^3.0.0"}}\n' >/proj/package.json
cat >/cfg/idea.vmoptions <<'V'
-Xmx2g
-Didea.config.path=/cfg/config
-Didea.system.path=/cfg/system
-Didea.plugins.path=/cfg/plugins
-Didea.log.path=/cfg/log
-Djb.consents.confirmation.enabled=false
-Djb.privacy.policy.text=<!--999.999-->
-Didea.trust.all.projects=true
-Didea.initially.ask.config=never
-Dide.show.tips.on.startup.default.value=false
V

# The server reaches the registries only through this proxy, which never answers.
perl -MIO::Socket::INET -e '$s = IO::Socket::INET->new(LocalPort => 3128, Listen => 256, ReuseAddr => 1) or die; push @c, $s->accept while 1' &
cat >/cfg/stall-lsp <<'S'
#!/bin/sh
export HTTPS_PROXY=http://127.0.0.1:3128 HTTP_PROXY=http://127.0.0.1:3128
# Only the language server session is recorded: a probe such as --file-types exits on its own, and
# tee would keep it waiting for input.
[ "$*" = --stdio ] || exec /target/release/versionlens-lsp "$@"
tee -a /cfg/lsp-in.log | /target/release/versionlens-lsp --stdio
S
chmod +x /cfg/stall-lsp

export DISPLAY=:99 IDEA_VM_OPTIONS=/cfg/idea.vmoptions VERSIONLENS_LSP=/cfg/stall-lsp
Xvfb :99 -screen 0 1600x1000x24 >/dev/null 2>&1 &
/idea/bin/idea.sh /proj >"$out/idea-stdout.log" 2>&1 &

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
	cp /cfg/log/idea.log /cfg/lsp-in.log "$out/" 2>/dev/null || true
	echo "jetbrains-gui: $1" >&2
	exit 1
}

wait_for 300 xdotool search --class jetbrains-idea || fail "the IDE window never appeared"
# A file passed to a second launch before /proj is open becomes a project of its own and
# disposes /proj, so the second launch waits for the IDE to list /proj as open.
wait_for 300 grep -q 'Project proj was added to the list of open projects' /cfg/log/idea.log ||
	fail "the IDE never opened /proj"
# The running IDE opens a file passed to a second launch, which starts the server; Escape
# dismisses the first-run popups.
open_manifest() {
	xdotool key Escape
	timeout 60 /idea/bin/idea.sh /proj/package.json >>"$out/idea-stdout.log" 2>&1 || true
	sleep 5
	pgrep -f /target/release/versionlens-lsp
}
wait_for 10 open_manifest || fail "opening package.json never started versionlens-lsp"
sleep 5
import -window root "$out/editor.png"

# The VersionLens toolbar sits under the editor tabs; its spinner turns while lookups are pending.
i=0
while [ $i -lt 40 ]; do
	import -window root -crop 1000x40+300+80 "$out/toolbar-$(printf %02d $i).png"
	i=$((i + 1))
	sleep 0.25
done
frames=$(md5sum "$out"/toolbar-*.png | cut -d' ' -f1 | sort -u | wc -l)
cp /cfg/log/idea.log "$out/"
if grep -q 'Plugin to blame: VersionLens' "$out/idea.log"; then
	fail "the IDE blamed VersionLens; see $out/idea.log"
fi
[ "$frames" -gt 1 ] || fail "the toolbar did not change across 40 frames; see $out/editor.png"

# Runs an action by its name through Find Action, from the editor.
# Xvfb runs no window manager, so a closed dialog leaves the X focus nowhere; the IDE frame, named
# after the project, gets it back first.
action() {
	for w in $(xdotool search --name '^proj'); do xdotool windowfocus --sync "$w" 2>/dev/null || true; done
	xdotool mousemove 600 400 click 1
	sleep 1
	xdotool key ctrl+shift+a
	sleep 2
	xdotool type --delay 30 "$1"
	sleep 3
	shot "find-$2"
	xdotool key Return
	sleep 3
}
shot() { import -window root "$out/$1.png"; }
# Types $1 into the focused dialog field and confirms it.
answer() {
	xdotool type --delay 30 "$1"
	sleep 1
	shot "$2"
	xdotool key Return
	sleep 3
}
# The server pulls the section again after each change; "u:p" is dTpw in base64.
pulled() { grep -c "$1" /cfg/lsp-in.log || true; }

# Add: the URL input starts at https://, the kind dialog's default button is Basic, then the user
# and the password, and an information message that Return closes.
action "Add URL Authentication" add
# The field starts with https:// selected, so the URL is typed whole.
xdotool key ctrl+a
answer "https://npm.example.com/" add-url
shot add-kind
xdotool key Return
sleep 3
answer u add-user
answer p add-password
shot add-saved
xdotool key Return
wait_for 30 grep -q '"https://npm.example.com/":"Basic dTpw"' /cfg/lsp-in.log ||
	fail "the server never received the stored Basic header"

# Remove: the picker lists the stored URL; Down selects it and Return confirms.
seen=$(pulled '"authorization":{}')
action "Remove URL Authentication" remove
shot remove-picker
xdotool key Down
sleep 1
shot remove-picked
xdotool key Return
sleep 3
shot remove-done
xdotool key Return
gone() { [ "$(pulled '"authorization":{}')" -gt "$seen" ]; }
wait_for 30 gone || fail "the server still has the header after Remove URL Authentication"
action "Show VersionLens Log" log
sleep 3
shot trace
cp /cfg/log/idea.log /cfg/lsp-in.log "$out/"
if grep -q 'Plugin to blame: VersionLens' "$out/idea.log"; then
	fail "the IDE blamed VersionLens; see $out/idea.log"
fi
echo "jetbrains-gui: server started, $frames distinct toolbar frames, URL authentication added" \
	"through its dialogs and removed through its picker, no errors blamed on VersionLens"
