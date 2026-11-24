# Zed's runtime libraries and Mesa's software Vulkan driver on top of the JetBrains GUI container,
# for `just zed-gui`: Zed draws into Xvfb through lavapipe, so no window reaches the desktop.
FROM versionlens-jetbrains-gui

RUN apt-get update \
	&& apt-get install -y --no-install-recommends libasound2t64 libvulkan1 libwayland-client0 \
		libx11-xcb1 libxcb1 libxkbcommon-x11-0 libxkbcommon0 mesa-vulkan-drivers \
	&& rm -rf /var/lib/apt/lists/*
