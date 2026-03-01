/**
 * Interface for a component that handles authentication and authorization for external services.
 */
export interface IAuthorizer {
  /**
   * Prompts the user for credentials for a specific URL.
   * @param url The authorization URL.
   * @param requestUrl The original requested URL that triggered the authorization.
   * @returns A promise resolving to true if credentials were successfully acquired, otherwise false.
   */
  getCredentials(url: string, requestUrl: string): Promise<boolean>;

  /**
   * Retries acquiring credentials for a specific URL, usually after a failed attempt.
   * @param url The authorization URL.
   * @returns A promise resolving to true if credentials were successfully acquired, otherwise false.
   */
  retryCredentials(url: string): Promise<boolean>;

  /**
   * Gets an authorization token for a specific URL.
   * @param url The authorization URL.
   * @returns A promise resolving to the token string, or undefined if no token is available.
   */
  getToken(url: string): Promise<string | undefined>;

  /**
   * Checks if an authorization URL is configured or supported.
   * @param url The URL to check.
   * @returns True if the URL is supported, otherwise false.
   */
  hasAuthorizationUrl(url: string): boolean;

  /**
   * Gets the canonical authorization URL for a given service URL.
   * @param url The service URL.
   * @returns The resolved authorization URL.
   */
  getAuthorizationUrl(url: string): string;
}