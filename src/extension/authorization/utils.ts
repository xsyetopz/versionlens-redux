import {
  type UrlAuthenticationData,
  AuthenticationScheme,
  UrlAuthenticationStatus
} from '#extension/authorization';

/**
 * Creates a UrlAuthenticationData object.
 * @param url The authorized URL.
 * @param scheme The authentication scheme.
 * @param label The display label for the authentication.
 * @param status The current authentication status.
 * @returns A new UrlAuthenticationData object.
 */
export function createUrlAuthData(
  url: string,
  scheme: AuthenticationScheme,
  label: string,
  status: UrlAuthenticationStatus
): UrlAuthenticationData {
  const parsedUrl = new URL(url);
  return {
    url,
    scheme,
    protocol: parsedUrl.protocol,
    label,
    status
  };
}

/**
 * Creates an empty UrlAuthenticationData object representing a cancelled state.
 * @param url The URL for which authentication was cancelled.
 * @returns A new UrlAuthenticationData object with NotSet scheme and UserCancelled status.
 */
export function createEmptyUrlAuthData(url: string): UrlAuthenticationData {
  const parsedUrl = new URL(url);
  return {
    url,
    scheme: AuthenticationScheme.NotSet,
    protocol: parsedUrl.protocol,
    status: UrlAuthenticationStatus.UserCancelled
  };
}