import {
  PackageDependency,
  PackageDescriptorType,
  SuggestionCategory,
  SuggestionTypes,
  TPackageNameVersion,
  TPackageResource,
  TPackageSuggestion,
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

export function createSuggestion(
  name: string,
  category: SuggestionCategory,
  version: string,
  type: SuggestionTypes
): TPackageSuggestion {
  return { name, category, version, type };
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