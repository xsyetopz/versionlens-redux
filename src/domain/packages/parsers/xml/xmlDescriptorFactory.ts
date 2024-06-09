import { PackageDescriptorType } from "../definitions/ePackageDescriptorType";
import { TPackageNameDescriptor, TPackageVersionDescriptor } from "../definitions/tPackageTypeDescriptors";
import { createProjectVersionTypeDesc } from "../json/jsonPackageTypeFactory";
import { PackageDescriptor } from "../packageDescriptor";
import { createPackageVersionDesc } from "../packageDescriptorTypeUtils";
import { XmlNode } from "./xmlParser";

export function createNameDescFromXmlElem(keyNode: XmlNode): TPackageNameDescriptor {
  const nameRange = {
    start: keyNode.tagOpenStart,
    end: keyNode.tagOpenStart
  };

  return {
    type: PackageDescriptorType.name,
    name: keyNode.name,
    nameRange
  };
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