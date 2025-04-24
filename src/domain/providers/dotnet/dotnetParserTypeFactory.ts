import {
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  type XmlNode,
  createPackageNameDesc,
  createPackageVersionDesc
} from '#domain/parsers';

export function createNameDescFromXmlAttr(node: XmlNode): PackageNameDescriptor {
  const includeAttr = node.attributes.include || node.attributes.update;
  if (!includeAttr) return undefined;

  const nameRange = {
    start: node.tagOpenStart,
    end: node.tagOpenStart
  };

  return createPackageNameDesc(includeAttr.value, nameRange);
}

export function createVersionDescFromXmlAttr(keyNode: XmlNode): PackageVersionDescriptor {
  const versionAttr = keyNode.attributes.version || keyNode.attributes.versionoverride;
  if (!versionAttr) return undefined;

  const versionRange = {
    start: versionAttr.start,
    end: versionAttr.end,
  };

  return createPackageVersionDesc(versionAttr.value, versionRange);
}

export function createSdkNameDescFromXmlAttr(node: XmlNode): PackageNameDescriptor {
  const nameAttr = node.attributes.name;
  if (!nameAttr) return undefined;

  const nameRange = {
    start: node.tagOpenStart,
    end: node.tagOpenStart
  };

  return createPackageNameDesc(nameAttr.value, nameRange);
}

export function createBlankVersionDescFromXmlAttr(node: XmlNode): PackageVersionDescriptor {
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