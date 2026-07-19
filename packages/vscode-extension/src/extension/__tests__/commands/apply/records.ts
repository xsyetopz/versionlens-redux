function clearRecord(record: Record<string, unknown>): void {
  for (const key of Object.keys(record)) {
    Reflect.deleteProperty(record, key);
  }
}

export { clearRecord };
