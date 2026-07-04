export type NativeSessionConfig = {
	cacheDurationMinutes?: number;
	enabledProviders?: string[];
	http?: NativeHttpConfig;
	providers?: NativeProviderSettings;
	showPrereleases: boolean;
	showSuggestionStats?: boolean;
	showVulnerabilities?: boolean;
	suggestionIndicators?: NativeSuggestionIndicators;
};

export type NativeProviderSettings = {
	dependencyProperties?: NativeDependencyPropertyConfig[];
	filePatterns?: NativeFilePatternConfig[];
	prereleaseTagFilters?: NativePrereleaseTagFilter[];
	providerCache?: NativeProviderCacheConfig[];
	providerHttp?: NativeProviderHttpConfig[];
	registryUrls?: NativeRegistryUrl[];
};

export type NativeDependencyPropertyConfig = {
	ecosystem: string;
	properties: string[];
	provider?: string;
};

export type NativeFilePatternConfig = {
	ecosystem: string;
	pattern: string;
};

export type NativeSuggestionIndicators = {
	build?: string;
	directory?: string;
	error?: string;
	latest?: string;
	matched?: string;
	noMatch?: string;
	satisfiesLatest?: string;
	updateable?: string;
	updateableVulnerable?: string;
};

export type NativeRegistryUrl = {
	ecosystem: string;
	url: string;
};

export type NativePrereleaseTagFilter = {
	ecosystem: string;
	tags: string[];
};

export type NativeProviderHttpConfig = {
	ecosystem: string;
	strictSsl?: boolean;
};

export type NativeProviderCacheConfig = {
	cacheDurationMinutes: number;
	ecosystem: string;
};

export type NativeHttpConfig = {
	authHeaders?: NativeHttpHeader[];
	ca?: string;
	caFile?: string;
	cert?: string;
	certFile?: string;
	key?: string;
	keyFile?: string;
	proxy?: string;
	strictSsl?: boolean;
	timeoutMs?: number;
};

export type NativeHttpHeader = {
	name: string;
	url?: string;
	value: string;
};
