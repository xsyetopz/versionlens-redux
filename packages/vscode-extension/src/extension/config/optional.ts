const optionalProperty = <Key extends PropertyKey, Value>(
	key: Key,
	value: Value | undefined,
): Partial<Record<Key, Value>> => {
	const result: Partial<Record<Key, Value>> = {};
	if (value !== undefined) {
		result[key] = value;
	}
	return result;
};

export { optionalProperty };
