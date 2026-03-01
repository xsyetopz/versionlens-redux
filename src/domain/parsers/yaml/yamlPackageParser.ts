import {
  type YamlParserOptions,
  type YamlTypeDescriptorHandler,
  createNameDescFromYamlNode,
  createPackageParentDescType,
  findByPath,
  getPackageProjectVersionDesc,
  isNodeQuoted,
  PackageDescriptor
} from '#domain/parsers';
import type { KeyDictionary } from '#domain/utils';
import {
  type Document,
  type Pair,
  type ParsedNode,
  type YAMLMap,
  isMap,
  isScalar,
  parseDocument
} from 'yaml';
import { findPair } from 'yaml/util';

/**
 * Parses package descriptors from a YAML string.
 * @param yaml The YAML content to parse.
 * @param options Parser options.
 * @returns An array of Identified package descriptors.
 */
export function parsePackagesYaml(
  yaml: string,
  options: YamlParserOptions
): Array<PackageDescriptor> {
  const yamlDoc = parseDocument(yaml);
  if (!yamlDoc || !yamlDoc.contents || yamlDoc.errors.length > 0) return [];

  return parsePackageNodes(yamlDoc, options);
}

/**
 * Parses package nodes from a root YAML document.
 * @param rootNode The root YAML document.
 * @param options Parser options.
 * @returns An array of Identified package descriptors.
 */
function parsePackageNodes(
  rootNode: Document.Parsed<ParsedNode>,
  options: YamlParserOptions
): PackageDescriptor[] {
  const matchedDependencies: Array<PackageDescriptor> = [];
  const { includePropNames, complexTypeHandlers } = options;

  for (const incPropName of includePropNames) {
    const segments = incPropName.split(".");
    const found = findByPath(rootNode, segments);
    if (found.length === 0) continue;

    if (incPropName === 'version') {
      const versionDesc = getPackageProjectVersionDesc(rootNode.contents as YAMLMap)
      versionDesc && matchedDependencies.push(versionDesc);
      continue;
    }

    const children = descendChildNodes(incPropName, found, complexTypeHandlers);
    matchedDependencies.push.apply(matchedDependencies, children);
  }

  return matchedDependencies;
}

/**
 * Descends into child nodes to extract package descriptors.
 * @param path The current path in the YAML tree.
 * @param pairs The pairs to process.
 * @param complexTypeHandlers Map of complex type handlers.
 * @returns An array of Identified package descriptors.
 */
function descendChildNodes(
  path: string,
  pairs: Array<Pair<string, any>>,
  complexTypeHandlers: KeyDictionary<YamlTypeDescriptorHandler>
): Array<PackageDescriptor> {
  const matchedDependencies: Array<PackageDescriptor> = [];
  const createVersionDescFromYamlNode = complexTypeHandlers['version'];

  for (const pair of pairs) {
    const { key: keyNode, value: valueNode } = pair;
    const isScalarValue = isScalar(valueNode);
    const isQuotedType = isScalarValue && isNodeQuoted(valueNode);

    // parse string properties
    if (isScalarValue) {
      // create the name descriptor
      const nameDesc = createNameDescFromYamlNode(keyNode);
      // create the version descriptor
      const versionDesc = createVersionDescFromYamlNode(valueNode, isQuotedType);
      if (!versionDesc) continue;

      // create the parent path desc
      const parentDesc = createPackageParentDescType(path);
      // create the package descriptor
      const packageDesc = new PackageDescriptor([nameDesc, versionDesc, parentDesc]);
      // add the package desc to the matched array
      matchedDependencies.push(packageDesc);
      continue;
    }

    // parse complex properties
    if (isMap(valueNode)) {
      const map = valueNode as YAMLMap;

      // create the package descriptor
      const packageDesc = new PackageDescriptor([]);

      for (const typeName in complexTypeHandlers) {
        if (map.has(typeName) === false) continue;

        const pair = findPair(map.items, typeName);
        if (!pair) continue;

        // get the type desc
        const handler = complexTypeHandlers[typeName];

        // add the handled type to the package desc
        const isScalarValue = isScalar(pair.value)
        const isQuotedType = isScalarValue && isNodeQuoted(pair.value);
        const typeDesc = handler(pair.value, isQuotedType);
        if (!typeDesc) continue;

        packageDesc.addType(typeDesc);
      }

      // skip when no types were added
      if (packageDesc.typeCount === 0) continue;

      // add the name descriptor
      const nameDesc = createNameDescFromYamlNode(keyNode);
      packageDesc.addType(nameDesc);

      // add the parent path desc
      packageDesc.addType(createPackageParentDescType(path));

      // add the package desc to the matched array
      matchedDependencies.push(packageDesc);
    }
  }

  return matchedDependencies;
}