const REGISTRY_URL_PREFIX = /^https?:\/\//u;

function normalizedRegistryUrl(value: string | undefined): string | undefined {
  const url = normalizedRequiredInput(value);
  let normalized: string | undefined;
  if (url && REGISTRY_URL_PREFIX.test(url)) {
    normalized = url;
  }
  return normalized;
}

function normalizedRequiredInput(
  value: string | undefined,
): string | undefined {
  const trimmed = value?.trim();
  let normalized: string | undefined;
  if (trimmed) {
    normalized = trimmed;
  }
  return normalized;
}

export { normalizedRegistryUrl, normalizedRequiredInput };
