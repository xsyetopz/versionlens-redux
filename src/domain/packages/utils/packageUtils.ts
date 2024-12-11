import type {
  PackageDependency,
  PackageSuggestion,
  SuggestionCategory,
  SuggestionTypes,
  TPackageNameVersion,
  TPackageResource
} from '#domain/packages';
import { PackageDescriptorType } from '#domain/parsers';

export function createPackageNameVersion(name: string, version: string): TPackageNameVersion {
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
): PackageSuggestion {
  return { name, category, version, type };
}

export function hasPackageDepsChanged(
  original: PackageDependency[],
  changed: PackageDependency[]
): boolean {
  if (original.length !== changed.length) return true;

  for (const dep of original) {

    if (dep.descriptors.hasType(PackageDescriptorType.ignoreChanges)) continue;

    const noChange = changed.some(
      other => other.packageEquals(dep)
    );

    if (noChange === false) return true;
  }

  return false;
}