export interface IAuthorizer {
  getConsent(url: string, requestUrl: string): Promise<boolean>;
  retryCredentials(url: string): Promise<boolean>;
  getToken(url: string): Promise<string | undefined>;
  hasAuthorizationUrl(url: string): boolean;
  getAuthorizationUrl(url: string): string;
}