export interface IAuthorizer {
  getConsent(url: string, requestUrl: string): Promise<boolean>;
  retryCredentials(url: string): Promise<boolean>;
  urlHasAuthConsent(url: string): boolean;
  getToken(url: string): Promise<string | undefined>;
  getRegistryAuthUrl(url: string): string;
}