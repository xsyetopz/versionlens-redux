import {
  PackageDependency,
  PackageDescriptorType,
  TPackageNameVersion,
  TPackageResource,
  TPackageTextRange
} from "domain/packages";

export function createDependencyRange(
  start: number,
  end: number
): TPackageTextRange {
  return {
    start,
    end
  }
}

export function createPackageNameVersion(
  name: string,
  version: string
): TPackageNameVersion {
  return {
    name,
    version
  }
}

export function createPackageResource(
  name: string,
  version: string,
  path: string
): TPackageResource {
  return {
    name,
    version,
    path
  }
}

export function hasPackageDepsChanged(
  original: PackageDependency[],
  changed: PackageDependency[]
): boolean {
  if (original.length !== changed.length) return true;

  for (const dep of original) {

    if (dep.packageDesc.hasType(PackageDescriptorType.ignoreChanges)) continue;

    const noChange = changed.some(
      other => other.packageEquals(dep)
    );

    if (noChange === false) return true;
  }

  return false;
}