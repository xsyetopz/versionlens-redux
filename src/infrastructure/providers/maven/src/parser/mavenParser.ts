import { PackageDescriptor, XmlDoc, XmlNode } from "domain/packages";
import { createNameDescFromXmlNodes, createVersionDescFromXmlNodes } from "./mavenParserTypeFactory";

export function parseMavenPackagesXml(
  xml: string,
  includePropertyNames: Array<string>
): Array<PackageDescriptor> {
  if (includePropertyNames.length === 0) return [];

  const document = new XmlDoc();
  document.parse(xml)
  if (document.errors.length > 0) return [];

  // get the project.properties nodes
  const propertyNodes = document.findPathsStartsWith("project.properties");

  return extractDependenciesFromNodes(document, propertyNodes, includePropertyNames);
}

export function extractDependenciesFromNodes(
  doc: XmlDoc,
  propertyNodes: XmlNode[],
  includePropNames: string[]
): Array<PackageDescriptor> {

  const matchedDependencies: Array<PackageDescriptor> = [];

  const includeNodes = includePropNames.map(n => doc.findExactPaths(n)).flat();

  for (const node of includeNodes) {

    const childNodes = doc.getChildren(node)
    if (childNodes.length === 0) continue;

    const packageDesc = mapChildrenToDescriptor(node, childNodes, propertyNodes);
    if (!packageDesc) continue;

    matchedDependencies.push(packageDesc)
  }

  return matchedDependencies;
}

export function mapChildrenToDescriptor(
  parentNode: XmlNode,
  childNodes: XmlNode[],
  propertyNodes: XmlNode[]
): PackageDescriptor {

  // get the name descriptor
  const nameDesc = createNameDescFromXmlNodes(parentNode, childNodes, propertyNodes);
  if (!nameDesc) return;

  // get the version descriptor
  const versionDesc = createVersionDescFromXmlNodes(childNodes, propertyNodes);
  if (!versionDesc) return;

  // create the package descriptor
  const packageDesc = new PackageDescriptor([nameDesc, versionDesc]);

  // return the package descriptor
  return packageDesc;
}