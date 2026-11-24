# VS Code, Xvfb, Bun, and the pinned Rust toolchain, for `just vscode-live` and
# `just vscode-bench`: VS Code draws into a virtual display, so no window reaches the desktop.
FROM rust:1.100.0-slim-trixie

ARG TARGETARCH
RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates curl unzip xauth xvfb \
	&& curl -fsSL -o /tmp/code.deb \
		"https://update.code.visualstudio.com/latest/linux-deb-$([ "$TARGETARCH" = amd64 ] && echo x64 || echo "$TARGETARCH")/stable" \
	&& apt-get install -y --no-install-recommends /tmp/code.deb \
	&& rm -rf /tmp/code.deb /var/lib/apt/lists/*
COPY --from=oven/bun:1.4.5-slim /usr/local/bin/bun /usr/local/bin/bun
