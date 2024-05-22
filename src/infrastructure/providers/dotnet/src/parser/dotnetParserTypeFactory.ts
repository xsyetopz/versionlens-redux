import {
  PackageDescriptorType,
  TPackageNameDescriptor,
  TPackageVersionDescriptor,
  XmlNode
} from "domain/packages";

export const noVersionAttr = '>=*.*.*';

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

  return {
    type: PackageDescriptorType.version,
    version: versionAttr.value,
    versionRange
  };
}

export function createBlankVersionDescFromXmlAttr(node: XmlNode): TPackageVersionDescriptor {
  const start = node.isSelfClosing ? node.tagCloseStart : node.tagOpenEnd - 1;
  const versionRange = {
    start,
    end: start,
  };

  return {
    type: PackageDescriptorType.version,
    version: noVersionAttr,
    versionRange
  };
}