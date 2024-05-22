import {
  PackageDescriptorType,
  TPackageNameDescriptor,
  TPackageVersionDescriptor,
  XmlNode,
  createPackageVersionDesc
} from "domain/packages";

export function createNameDescFromXmlAttr(node: XmlNode): TPackageNameDescriptor {
  const includeAttr = node.attributes.include || node.attributes.update;
  if (!includeAttr) return undefined;

  const nameRange = {
    start: node.tagOpenStart,
    end: node.tagOpenStart
  };

  return {
    type: PackageDescriptorType.name,
    name: includeAttr.value,
    nameRange
  };
}

export function createSdkNameDescFromXmlAttr(node: XmlNode): TPackageNameDescriptor {
  const nameAttr = node.attributes.name;
  if (!nameAttr) return undefined;

  const nameRange = {
    start: node.tagOpenStart,
    end: node.tagOpenStart
  };

  return {
    type: PackageDescriptorType.name,
    name: nameAttr.value,
    nameRange
  };
}

export function createVersionDescFromXmlAttr(keyNode: XmlNode): TPackageVersionDescriptor {
  const versionAttr = keyNode.attributes.version || keyNode.attributes.versionoverride;
  if (!versionAttr) return undefined;

  const versionRange = {
    start: versionAttr.start,
    end: versionAttr.end,
  };

  return createPackageVersionDesc(versionAttr.value, versionRange);
}

export function createBlankVersionDescFromXmlAttr(node: XmlNode): TPackageVersionDescriptor {
  const end = node.isSelfClosing ? node.tagCloseStart : node.tagOpenEnd - 1;
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