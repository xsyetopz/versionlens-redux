import type { XmlNode } from '#domain/parsers';
import {
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptor,
  createPackageNameDesc,
  createPackageVersionDesc,
  createProjectVersionTypeDesc
} from '#domain/parsers';

/**
 * Creates a package name descriptor from an XML element.
 * @param keyNode The XML node representing the element.
 * @returns A package name descriptor.
 */
export function createNameDescFromXmlElem(keyNode: XmlNode): PackageNameDescriptor {
  const nameRange = {
    start: keyNode.tagOpenStart,
    end: keyNode.tagOpenStart
  };

  return createPackageNameDesc(keyNode.name, nameRange);
}

/**
 * Creates a package version descriptor from an XML element.
 * @param keyNode The XML node representing the element.
 * @returns A package version descriptor.
 */
export function createVersionDescFromXmlElem(keyNode: XmlNode): PackageVersionDescriptor {
  const versionText = keyNode.text ?? '';
  const versionRange = {
    start: keyNode.tagOpenEnd,
    end: keyNode.tagCloseStart
  };
  return createPackageVersionDesc(versionText, versionRange);
}

/**
 * Creates a package descriptor for the project's own version from an XML element.
 * @param node The XML node.
 * @returns A package descriptor for the project version.
 */
export function createProjectVersionDescFromXmlElem(node: XmlNode): PackageDescriptor {
  const nameDesc = createNameDescFromXmlElem(node);
  const versionDesc = createVersionDescFromXmlElem(node);
  const projectVersionDesc = createProjectVersionTypeDesc();
  return new PackageDescriptor([
    nameDesc,
    versionDesc,
    projectVersionDesc
  ]);
}