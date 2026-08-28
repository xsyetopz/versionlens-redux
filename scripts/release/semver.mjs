const semverPattern =
  /^(?<major>0|[1-9]\d*)\.(?<minor>0|[1-9]\d*)\.(?<patch>0|[1-9]\d*)(?:-(?<prerelease>[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+(?<build>[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?$/u;
const numericIdentifierPattern = /^\d+$/u;

function fail(message) {
  throw new Error(message);
}

function parseSemver(value) {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.trim() !== value
  ) {
    fail(`invalid SemVer: ${JSON.stringify(value)}`);
  }
  const match = semverPattern.exec(value);
  if (!match) {
    fail(`invalid SemVer: ${JSON.stringify(value)}`);
  }
  const prerelease = match[4]?.split(".") ?? [];
  for (const identifier of prerelease) {
    if (
      numericIdentifierPattern.test(identifier) &&
      identifier.length > 1 &&
      identifier[0] === "0"
    ) {
      fail(
        `invalid SemVer numeric prerelease identifier: ${JSON.stringify(identifier)}`,
      );
    }
  }
  return {
    major: BigInt(match[1]),
    minor: BigInt(match[2]),
    patch: BigInt(match[3]),
    prerelease,
  };
}

function compareSemver(left, right) {
  for (const field of ["major", "minor", "patch"]) {
    if (left[field] !== right[field]) {
      if (left[field] < right[field]) {
        return -1;
      }
      return 1;
    }
  }
  return comparePrerelease(left.prerelease, right.prerelease);
}

function comparePrerelease(left, right) {
  if (left.length === 0 || right.length === 0) {
    if (left.length === right.length) {
      return 0;
    }
    if (left.length === 0) {
      return 1;
    }
    return -1;
  }
  const length = Math.max(left.length, right.length);
  for (let index = 0; index < length; index += 1) {
    const comparison = comparePrereleaseIdentifier(left[index], right[index]);
    if (comparison !== 0) {
      return comparison;
    }
  }
  return 0;
}

function comparePrereleaseIdentifier(left, right) {
  if (left === undefined || right === undefined) {
    if (left === right) {
      return 0;
    }
    if (left === undefined) {
      return -1;
    }
    return 1;
  }
  if (left === right) {
    return 0;
  }
  const leftNumeric = numericIdentifierPattern.test(left);
  const rightNumeric = numericIdentifierPattern.test(right);
  if (leftNumeric && rightNumeric) {
    if (BigInt(left) < BigInt(right)) {
      return -1;
    }
    return 1;
  }
  if (leftNumeric !== rightNumeric) {
    if (leftNumeric) {
      return -1;
    }
    return 1;
  }
  if (left < right) {
    return -1;
  }
  return 1;
}

function replaceExactly({ source, pattern, replacement, expectedCount, path }) {
  let count = 0;
  const output = source.replace(pattern, (...args) => {
    count += 1;
    if (typeof replacement === "function") {
      return replacement(...args);
    }
    return replacement;
  });
  if (count !== expectedCount) {
    fail(
      `${path}: expected ${expectedCount} version occurrence(s), found ${count}`,
    );
  }
  return output;
}

export { compareSemver, fail, parseSemver, replaceExactly };
