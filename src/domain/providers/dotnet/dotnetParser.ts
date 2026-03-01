import { PackageDescriptor, XmlDoc, createProjectVersionDescFromXmlElem } from '#domain/parsers';
import {
  createBlankVersionDescFromXmlAttr,
  createNameDescFromXmlAttr,
  createSdkNameDescFromXmlAttr,
  createVersionDescFromXmlAttr
} from '#domain/providers/dotnet';

/**
 * Parses a DotNet project file (XML) and extracts package dependencies.
 * @param xml The content of the XML file.
 * @param includePropertyNames The property names to include in the search.
 * @returns An array of Identified package descriptors.
 */
export function parseDotNetPackagesXml(
  xml: string,
  includePropertyNames: Array<string>
): Array<PackageDescriptor> {
  if (includePropertyNames.length === 0) return [];

  const document = new XmlDoc();
  document.parse(xml)
  if (document.errors.length > 0) return [];

  return parsePackageNodes(document, includePropertyNames);
}

/**
 * Parses package nodes from an XML document.
 * @param doc The XML document.
 * @param includePropNames The property names to include.
 * @returns An array of Identified package descriptors.
 */
export function parsePackageNodes(
  doc: XmlDoc,
  includePropNames: string[]
): Array<PackageDescriptor> {

  const matchedDependencies: Array<PackageDescriptor> = [];

  const includeNodes = includePropNames.map(n => doc.findExactPaths(n)).flat();

  for (const node of includeNodes) {
    // check for project version entries
    switch (node.name) {
      case 'Version':
      case 'AssemblyVersion':
        // add the package desc to the matched array
        matchedDependencies.push(createProjectVersionDescFromXmlElem(node));
        continue;
    }

    // get the name descriptor
    const nameDesc = node.name === "Sdk"
      ? createSdkNameDescFromXmlAttr(node)
      : createNameDescFromXmlAttr(node);
    if (!nameDesc) continue;

    // get the version descriptor
    const versionDesc = createVersionDescFromXmlAttr(node)
      || createBlankVersionDescFromXmlAttr(node);

    if (!versionDesc) continue;

    // create the package descriptor
    const packageDesc = new PackageDescriptor([nameDesc, versionDesc]);

    // add the package desc to the matched array
    matchedDependencies.push(packageDesc);
  }

  return matchedDependencies;
}