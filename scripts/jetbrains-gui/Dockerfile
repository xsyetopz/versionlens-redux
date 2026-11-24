# IntelliJ IDEA's runtime libraries, X tools, and colour emoji on top of the VS Code container, for
# `just jetbrains-gui`: the IDE draws into Xvfb, so no window reaches the desktop.
FROM versionlens-vscode

# The VS Code package adds its own apt source, which the update cannot verify here.
RUN rm -f /etc/apt/sources.list.d/vscode* \
	&& apt-get update \
	&& apt-get install -y --no-install-recommends fontconfig fonts-dejavu-core fonts-noto-color-emoji \
		imagemagick libxi6 libxrender1 libxtst6 perl procps x11-utils xdotool \
	&& rm -rf /var/lib/apt/lists/*
