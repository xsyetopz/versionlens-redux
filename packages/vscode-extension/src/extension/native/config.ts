export interface NativeSessionConfig {
  cacheDurationMinutes?: number;
  enabledProviders?: string[];
  http?: NativeHttpConfig;
  providers?: NativeProviderSettings;
  showPrereleases: boolean;
  showSuggestionStats?: boolean;
  showVulnerabilities?: boolean;
  suggestionIndicators?: NativeSuggestionIndicators;
}

export interface NativeProviderSettings {
  dependencyProperties?: NativeDependencyPropertyConfig[];
  filePatterns?: NativeFilePatternConfig[];
  prereleaseTagFilters?: NativePrereleaseTagFilter[];
  providerCache?: NativeProviderCacheConfig[];
  providerHttp?: NativeProviderHttpConfig[];
  registryUrls?: NativeRegistryUrl[];
}

export interface NativeDependencyPropertyConfig {
  ecosystem: string;
  properties: string[];
  provider?: string;
}

export interface NativeFilePatternConfig {
  ecosystem: string;
  pattern: string;
}

export interface NativeSuggestionIndicators {
  build?: string;
  directory?: string;
  error?: string;
  latest?: string;
  matched?: string;
  noMatch?: string;
  satisfiesLatest?: string;
  updateable?: string;
  updateableVulnerable?: string;
}

export interface NativeRegistryUrl {
  ecosystem: string;
  url: string;
}

export interface NativePrereleaseTagFilter {
  ecosystem: string;
  tags: string[];
}

export interface NativeProviderHttpConfig {
  ecosystem: string;
  strictSsl?: boolean;
}

export interface NativeProviderCacheConfig {
  cacheDurationMinutes?: number;
  ecosystem: string;
}

export interface NativeHttpConfig {
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
}

export interface NativeHttpHeader {
  name: string;
  url?: string;
  value: string;
}
