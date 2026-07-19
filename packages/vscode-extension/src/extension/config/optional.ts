function optionalProperty<Key extends PropertyKey, Value>(
  key: Key,
  value: Value | undefined,
): Partial<Record<Key, Value>> {
  if (value === undefined) {
    return {};
  }
  return { [key]: value } as Record<Key, Value>;
}

export { optionalProperty };
