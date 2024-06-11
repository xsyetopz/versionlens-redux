import {
  PackageDescriptor,
  TPackageNameDescriptor,
  TPackageVersionDescriptor,
  XmlNode,
  createPackageNameDesc,
  createPackageVersionDesc,
  createProjectVersionTypeDesc
} from '#domain/packages';

export function createNameDescFromXmlElem(keyNode: XmlNode): TPackageNameDescriptor {
  const nameRange = {
    start: keyNode.tagOpenStart,
    end: keyNode.tagOpenStart
  };

  return createPackageNameDesc(keyNode.name, nameRange);
}

export function createVersionDescFromXmlElem(keyNode: XmlNode): TPackageVersionDescriptor {
  const versionText = keyNode.text ?? '';
  const versionRange = {
    start: keyNode.tagOpenEnd,
    end: keyNode.tagCloseStart
  };
  return createPackageVersionDesc(versionText, versionRange);
}

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