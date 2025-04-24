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
import { KeyDictionary } from '#domain/utils';
import { Document, isMap, isScalar, Pair, ParsedNode, parseDocument, YAMLMap } from 'yaml';
import { findPair } from 'yaml/util';

export function parsePackagesYaml(
  yaml: string,
  options: YamlParserOptions
): Array<PackageDescriptor> {
  const yamlDoc = parseDocument(yaml)
  if (!yamlDoc || !yamlDoc.contents || yamlDoc.errors.length > 0) return [];

  return parsePackageNodes(yamlDoc, options);
}

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
      matchedDependencies.push(getPackageProjectVersionDesc(rootNode.contents as YAMLMap));
      continue;
    }

    const children = descendChildNodes(incPropName, found, complexTypeHandlers);
    matchedDependencies.push.apply(matchedDependencies, children);
  }

  return matchedDependencies;
}

function descendChildNodes(
  path: string,
  pairs: Array<Pair<string, any>>,
  complexTypeHandlers: KeyDictionary<YamlTypeDescriptorHandler>
): Array<PackageDescriptor> {
  const matchedDependencies: Array<PackageDescriptor> = [];
  const createVersionDescFromYamlNode = complexTypeHandlers['version']

  for (const pair of pairs) {
    const { key: keyNode, value: valueNode } = pair;
    const isScalarValue = isScalar(valueNode)
    const isQuotedType = isScalarValue && isNodeQuoted(valueNode);

    // parse string properties
    if (isScalarValue) {
      // create the name descriptor
      const nameDesc = createNameDescFromYamlNode(keyNode);
      // create the version descriptor
      const versionDesc = createVersionDescFromYamlNode(valueNode, isQuotedType);
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