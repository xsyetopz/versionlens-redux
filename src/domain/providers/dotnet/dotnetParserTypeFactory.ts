import {
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  type XmlNode,
  createPackageNameDesc,
  createPackageVersionDesc
} from '#domain/parsers';

/**
 * Creates a package name descriptor from an XML attribute (Include or Update).
 * @param node The XML node.
 * @returns A package name descriptor or undefined.
 */
export function createNameDescFromXmlAttr(node: XmlNode): PackageNameDescriptor | undefined {
  const includeAttr = node.attributes.include || node.attributes.update;
  if (!includeAttr) return undefined;

  const nameRange = {
    start: node.tagOpenStart,
    end: node.tagOpenStart
  };

  return createPackageNameDesc(includeAttr.value, nameRange);
}

/**
 * Creates a package version descriptor from an XML attribute (Version or VersionOverride).
 * @param keyNode The XML node.
 * @returns A package version descriptor or undefined.
 */
export function createVersionDescFromXmlAttr(keyNode: XmlNode): PackageVersionDescriptor | undefined {
  const versionAttr = keyNode.attributes.version || keyNode.attributes.versionoverride;
  if (!versionAttr) return undefined;

  const versionRange = {
    start: versionAttr.start,
    end: versionAttr.end,
  };

  return createPackageVersionDesc(versionAttr.value, versionRange);
}

/**
 * Creates a package name descriptor for an Sdk attribute.
 * @param node The XML node.
 * @returns A package name descriptor or undefined.
 */
export function createSdkNameDescFromXmlAttr(node: XmlNode): PackageNameDescriptor | undefined {
  const nameAttr = node.attributes.name;
  if (!nameAttr) return undefined;

  const nameRange = {
    start: node.tagOpenStart,
    end: node.tagOpenStart
  };

  return createPackageNameDesc(nameAttr.value, nameRange);
}

/**
 * Creates a blank package version descriptor for an XML node that lacks a version attribute.
 * @param node The XML node.
 * @returns A package version descriptor with a wildcard version and appropriate prepend/append text.
 */
export function createBlankVersionDescFromXmlAttr(node: XmlNode): PackageVersionDescriptor {
  const end = node.isSelfClosing ? node.tagCloseStart! : node.tagOpenEnd - 1;
  const versionRange = {
    start: end,
    end,
  };

  let versionPrepend = "";
  let versionAppend = '"';

  const attrKeys = Object.keys(node.attributes);
  if (attrKeys.length > 0) {
    const lastAttrKey = attrKeys[attrKeys.length - 1];
    const prependSpace = end - node.attributes[lastAttrKey].end == 1
    versionPrepend = prependSpace ? " " : ""
    versionPrepend += 'Version="'
  }

  if (node.isSelfClosing) versionAppend += ' ';

  return createPackageVersionDesc(
    "*",
    versionRange,
    versionPrepend,
    versionAppend
  );
}