import type { CachingOptions } from '#domain/caching';
import type { HttpOptions, JsonClientResponse } from '#domain/clients';
import type { RubyConfig, RubyGitHubClient, RubyHttpClient, RubySuggestionResolver } from '#domain/providers/ruby';
import { nameOf } from '#domain/utils';

/**
 * Feature keys used for Ruby configuration.
 */
export enum RubyFeatures {
  Caching = 'ruby.caching',
  Http = 'ruby.http',
  ApiUrl = 'ruby.apiUrl',
  FilePatterns = 'ruby.files',
  OnSaveChangesTask = 'ruby.onSaveChanges',
  PrereleaseTagFilter = 'ruby.prereleaseTagFilter',
}

/**
 * Defines the services provided by the Ruby provider.
 */
export interface IRubyServices {
  /**
   * Caching options for Ruby.
   */
  rubyCachingOpts: CachingOptions;
  /**
   * HTTP options for Ruby.
   */
  rubyHttpOpts: HttpOptions;
  /**
   * Configuration for Ruby.
   */
  rubyConfig: RubyConfig;
  /**
   * HTTP client for fetching from Ruby.
   */
  rubyHttpClient: RubyHttpClient;
  /**
   * GitHub client for fetching from GitHub.
   */
  rubyGithubClient: RubyGitHubClient;
  /**
   * Resolver for Ruby suggestions.
   */
  rubySuggestionResolver: RubySuggestionResolver;
}

/**
 * Service name constant for Ruby services.
 */
export const RubyService = nameOf<IRubyServices>()

/**
 * Represents the JSON response for Ruby package versions.
 */
export type RubyHttpClientResponse = JsonClientResponse<string[]>
