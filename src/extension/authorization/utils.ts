import {
  type UrlAuthenticationData,
  AuthenticationScheme,
  UrlAuthenticationStatus
} from '#extension/authorization';

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

export function createEmptyUrlAuthData(url: string): UrlAuthenticationData {
  const parsedUrl = new URL(url);
  return {
    url,
    scheme: AuthenticationScheme.NotSet,
    protocol: parsedUrl.protocol,
    label: null,
    status: UrlAuthenticationStatus.UserCancelled
  };
}