import { loadNative } from "./native/module.ts";
import { cacheDurationMinutes } from "./session/cache.ts";
import {
	configuredEnabledProviders,
	configuredShowVulnerabilities,
	reloadConfigurationState,
} from "./session/flags.ts";
import { httpConfig } from "./session/http.ts";
import { suggestionIndicators } from "./session/indicators.ts";
import {
	dependencyProperties,
	filePatterns,
	prereleaseTagFilters,
	providerCacheConfigs,
	providerHttpConfigs,
	registryUrls,
} from "./session/providers.ts";
import type { ExtensionState } from "./state.ts";

export {
	cacheDurationMinutes,
	configuredEnabledProviders,
	configuredShowVulnerabilities,
	dependencyProperties,
	filePatterns,
	httpConfig,
	prereleaseTagFilters,
	registryUrls,
	reloadConfigurationState,
	suggestionIndicators,
};

export async function recreateSession(state: ExtensionState) {
	state.session?.disposeSession();
	const extensionPath = state.context?.extensionPath;
	if (!extensionPath) {
		return;
	}

	const indicators = suggestionIndicators();
	const enabledProviders = configuredEnabledProviders();
	const cacheDuration = cacheDurationMinutes();
	const showVulnerabilities = configuredShowVulnerabilities();
	state.session = loadNative(extensionPath).createSession({
		...(cacheDuration === undefined
			? {}
			: { cacheDurationMinutes: cacheDuration }),
		...(enabledProviders ? { enabledProviders } : {}),
		http: await httpConfig(state),
		providers: {
			dependencyProperties: dependencyProperties(),
			filePatterns: filePatterns(),
			prereleaseTagFilters: prereleaseTagFilters(),
			providerCache: providerCacheConfigs(),
			providerHttp: providerHttpConfigs(),
			registryUrls: registryUrls(),
		},
		showPrereleases: state.flags.showPrereleases,
		showSuggestionStats: state.flags.showSuggestionStats,
		...(showVulnerabilities === undefined ? {} : { showVulnerabilities }),
		...(indicators ? { suggestionIndicators: indicators } : {}),
	});
}
