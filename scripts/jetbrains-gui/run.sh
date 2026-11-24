#!/bin/sh
# Opens a package.json in IntelliJ IDEA under Xvfb with the VersionLens plugin installed, while a
# proxy that accepts and never answers keeps every lookup pending. It fails when the IDE blames
# the plugin for an error or a freeze, when the server never starts, or when the editor toolbar
# stays still while lookups are pending. Screenshots and the IDE log go to the output directory.
# Usage: run.sh <plugin zip> <output directory>, from the repository root; the IDE is the release
# the plugin builds against.
set -eu
zip=$1 out=$2
version=$(sed -n 's/.*intellijIdea("\(.*\)").*/\1/p' packages/jetbrains-plugin/build.gradle.kts)
[ -n "$version" ] || { echo "jetbrains-gui: no intellijIdea version in build.gradle.kts" >&2; exit 1; }
mkdir -p "$out" /idea /cfg/config /cfg/system /cfg/plugins /proj/.idea
rm -f "$out"/*.png "$out"/*.log

# The volume keeps the IDE between runs; a new pinned release replaces it.
if ! grep -qs "\"version\": \"$version\"" /idea/product-info.json; then
	rm -rf /idea/*
	arch=$([ "$(uname -m)" = aarch64 ] && echo -aarch64 || true)
	curl -fsSL "https://download.jetbrains.com/idea/idea-$version$arch.tar.gz" |
		tar xz -C /idea --strip-components=1
fi
unzip -qo "$zip" -d /cfg/plugins

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
exec /target/release/versionlens-lsp "$@"
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
	cp /cfg/log/idea.log "$out/" 2>/dev/null || true
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
echo "jetbrains-gui: server started, $frames distinct toolbar frames, no errors blamed on VersionLens"
