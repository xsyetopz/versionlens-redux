import exceptions from "./exceptions.json" with { type: "json" };

const REQUIRED_METADATA = ["reason", "owner", "control", "review"];

function locationKey(location) {
  return `${location.path}:${location.owner}:${location.role}`;
}

const fingerprintForCategory = {
  duplicateLogic: (finding) =>
    [
      `${finding.firstPath}:${finding.firstName}`,
      `${finding.secondPath}:${finding.secondName}`,
    ]
      .toSorted()
      .join("|"),
  overqualifiedPaths: (finding) =>
    `${finding.path}:${finding.kind}:${finding.qualified}`,
  oversizedFunctions: (finding) =>
    `${finding.path}:${finding.name}:${finding.parameterCount}`,
  oversizedShapes: (finding) =>
    `${finding.path}:${finding.name}:${finding.fieldCount}`,
  passThroughWrappers: (finding) =>
    `${finding.path}:${finding.name}:${finding.callee}`,
  repeatedComplexTypes: (finding) =>
    `${finding.typeText}|${finding.locations.map(locationKey).toSorted().join("|")}`,
  suppressedParameters: (finding) =>
    `${finding.path}:${finding.functionName}:${finding.parameterName}`,
  unusedParameters: (finding) =>
    `${finding.path}:${finding.functionName}:${finding.parameterName}`,
};

function validateException(record, seen) {
  if (!fingerprintForCategory[record.category]) {
    throw new Error(
      `quality exception has unknown category: ${record.category}`,
    );
  }
  if (!(record.fingerprint && typeof record.fingerprint === "string")) {
    throw new Error("quality exception is missing a fingerprint");
  }
  for (const field of REQUIRED_METADATA) {
    if (!(record[field] && typeof record[field] === "string")) {
      throw new Error(
        `quality exception ${record.category}:${record.fingerprint} is missing ${field}`,
      );
    }
  }
  const key = `${record.category}:${record.fingerprint}`;
  if (seen.has(key)) {
    throw new Error(`duplicate quality exception: ${key}`);
  }
  seen.add(key);
}

function indexedExceptions() {
  const indexed = new Map();
  const seen = new Set();
  for (const record of exceptions) {
    validateException(record, seen);
    const category = indexed.get(record.category) ?? new Map();
    category.set(record.fingerprint, record);
    indexed.set(record.category, category);
  }
  return indexed;
}

export function applyExceptions(result) {
  const indexed = indexedExceptions();
  const applied = [];
  const remaining = {};
  for (const [category, findings] of Object.entries(result)) {
    const fingerprint = fingerprintForCategory[category];
    if (!fingerprint) {
      throw new Error(`unsupported finding category: ${category}`);
    }
    const categoryExceptions = indexed.get(category) ?? new Map();
    remaining[category] = findings.filter((finding) => {
      const record = categoryExceptions.get(fingerprint(finding));
      if (!record) {
        return true;
      }
      applied.push(record);
      categoryExceptions.delete(record.fingerprint);
      return false;
    });
  }

  const stale = [...indexed.values()].flatMap((category) => [
    ...category.values(),
  ]);
  if (stale.length > 0) {
    throw new Error(
      `stale quality exception(s):\n${stale
        .map((record) => `- ${record.category}:${record.fingerprint}`)
        .join("\n")}`,
    );
  }
  return { applied, remaining };
}
