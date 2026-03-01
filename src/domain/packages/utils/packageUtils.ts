import type {
  PackageDependency,
  PackageNameVersion,
  PackageResource,
  PackageSuggestion,
  SuggestionCategory,
  SuggestionTypes
} from '#domain/packages';
import { PackageDescriptorType } from '#domain/parsers';

/**
 * Creates a PackageNameVersion object.
 * @param name The package name.
 * @param version The package version.
 * @returns A package name and version object.
 */
export function createPackageNameVersion(name: string, version: string): PackageNameVersion {
  return {
    name,
    version
  }
}

/**
 * Creates a PackageResource object.
 * @param name The package name.
 * @param version The package version.
 * @param path The path to the package file.
 * @returns A package resource object.
 */
export function createPackageResource(
  name: string,
  version: string,
  path: string
): PackageResource {
  return {
    name,
    version,
    path
  }
}

/**
 * Creates a PackageSuggestion object.
 * @param name The display name of the suggestion.
 * @param category The category of the suggestion.
 * @param version The suggested version string.
 * @param type The type of suggestion.
 * @returns A package suggestion object.
 */
export function createSuggestion(
  name: string,
  category: SuggestionCategory,
  version: string,
  type: SuggestionTypes
): PackageSuggestion {
  return { name, category, version, type };
}

/**
 * Checks if the list of package dependencies has changed between two states.
 * @param original The original list of dependencies.
 * @param changed The new list of dependencies.
 * @returns True if any dependencies have changed, otherwise false.
 */
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