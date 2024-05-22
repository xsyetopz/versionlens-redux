import { createParentDesc, PackageDescriptor } from 'domain/packages';
import { KeyDictionary } from 'domain/utils';
import { Document, isMap, Pair, ParsedNode, parseDocument, YAMLMap } from 'yaml';
import { findPair } from 'yaml/util';
import { TYamlPackageParserOptions } from './tYamlPackageParserOptions';
import { TYamlPackageTypeHandler } from './tYamlPackageTypeHandler';
import {
  createNameDescFromYamlNode,
  createVersionDescFromYamlNode
} from './yamlPackageTypeFactory';

export function parsePackagesYaml(
  yaml: string,
  options: TYamlPackageParserOptions
): Array<PackageDescriptor> {

  const yamlDoc = parseDocument(yaml)
  if (!yamlDoc || !yamlDoc.contents || yamlDoc.errors.length > 0) return [];

  return parsePackageNodes(yamlDoc, options);
}

function parsePackageNodes(
  rootNode: Document.Parsed<ParsedNode>,
  options: TYamlPackageParserOptions
): PackageDescriptor[] {
  const matchedDependencies: Array<PackageDescriptor> = [];
  const { includePropNames, complexTypeHandlers } = options;

  for (const incPropName of includePropNames) {
    const segments = incPropName.split(".");

    const node = rootNode.getIn(segments) as YAMLMap;
    if (!node) continue;

    const children = node instanceof Array
      ? descendChildNodes(incPropName, node, complexTypeHandlers)
      : descendChildNodes(incPropName, node.items, complexTypeHandlers);

    matchedDependencies.push.apply(matchedDependencies, children);
  }

  return matchedDependencies
}

function descendChildNodes(
  path: string,
  pairs: Array<Pair<any, any>>,
  complexTypeHandlers: KeyDictionary<TYamlPackageTypeHandler>
): Array<PackageDescriptor> {
  const matchedDependencies: Array<PackageDescriptor> = [];

  for (const pair of pairs) {
    const { key: keyNode, value: valueNode } = pair;
    const isQuotedType = isNodeQuoted(valueNode);
    const isStringType = valueNode.type === "PLAIN" || isQuotedType;

    // parse string properties
    if (isStringType) {

      // create the name descriptor
      const nameDesc = createNameDescFromYamlNode(keyNode);

      // create the version descriptor
      const versionDesc = createVersionDescFromYamlNode(
        valueNode,
        isQuotedType
      );

      // create the parent path desc
      const parentDesc = createParentDesc(path);

      // create the package descriptor
      const packageDesc = new PackageDescriptor([nameDesc, versionDesc, parentDesc]);

      // add the package desc to the matched array
      matchedDependencies.push(packageDesc);

      continue;
    }

    // parse complex properties
    if (isMap(valueNode)) {
      const map = valueNode as YAMLMap;
      const isQuotedType = isNodeQuoted(valueNode);

      // create the package descriptor
      const packageDesc = new PackageDescriptor([]);

      for (const typeName in complexTypeHandlers) {
        if (map.has(typeName)) {
          const pair = findPair(map.items, typeName);
          if (!pair) continue;

          // get the type desc
          const handler = complexTypeHandlers[typeName];

          // add the handled type to the package desc
          const typeDesc = handler(
            pair.value,
            isQuotedType
          );

          // skip types that are't fully defined
          if (!typeDesc) continue;

          packageDesc.addType(typeDesc);
        }
      }

      // skip when no types were added
      if (packageDesc.typeCount === 0) continue;

      // add the name descriptor
      const nameDesc = createNameDescFromYamlNode(keyNode);
      packageDesc.addType(nameDesc);

      // add the parent path desc
      packageDesc.addType(createParentDesc(path));

      // add the package desc to the matched array
      matchedDependencies.push(packageDesc);
    }

  }

  return matchedDependencies;
}

function isNodeQuoted(node: any) {
  return node.type === "QUOTE_SINGLE"
    || node.type === "QUOTE_DOUBLE";
}