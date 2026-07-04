const REGISTRY_URL_PREFIX = /^https?:\/\//;

export function normalizedAuthHeaderName(value: string | undefined) {
	return normalizedRequiredInput(value);
}

export function normalizedAuthHeaderValue(value: string | undefined) {
	return normalizedRequiredInput(value);
}

export function normalizedRegistryUrl(value: string | undefined) {
	const url = normalizedRequiredInput(value);
	return url && REGISTRY_URL_PREFIX.test(url) ? url : undefined;
}

function normalizedRequiredInput(value: string | undefined) {
	const trimmed = value?.trim();
	return trimmed ? trimmed : undefined;
}
