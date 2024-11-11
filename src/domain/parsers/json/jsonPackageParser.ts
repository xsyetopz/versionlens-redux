import {
  PackageDescriptor,
  TJsonPackageParserOptions,
  TJsonPackageTypeHandler,
  createNameDescFromJsonNode,
  createPackageParentDescType,
  createVersionDescFromJsonNode
} from '#domain/parsers';
import { KeyDictionary, Undefinable } from '#domain/utils';
import * as JsonC from 'jsonc-parser';

export function parsePackagesJson(
  json: string,
  options: TJsonPackageParserOptions
): Array<PackageDescriptor> {
  const jsonErrors: Array<JsonC.ParseError> = [];
  const rootNode = JsonC.parseTree(json, jsonErrors);

  const hasErrors = jsonErrors.length > 0;
  if (!rootNode || hasErrors) return [];

  const hasChildren = rootNode.children && rootNode.children.length > 0;
  if (hasChildren === false) return [];

  return parsePackageNodes(rootNode, options);
}

function parsePackageNodes(
  rootNode: JsonC.Node,
  options: TJsonPackageParserOptions
): Array<PackageDescriptor> {
  const matchedDependencies: Array<PackageDescriptor> = [];
  const { includePropNames, customDescriptorHandler, complexTypeHandlers } = options;

  for (const incPropName of includePropNames) {
    const found = findNodesAtLocation(rootNode, incPropName);
    if (!found || !found.node) continue;

    if (found.node instanceof Array) {
      const matched = descendChildNodes(found.path, found.node, complexTypeHandlers);
      matchedDependencies.push.apply(matchedDependencies, matched);
      continue;
    }

    const hasChildren = found.node.children && found.node.children.length > 0;
    if (hasChildren) {
      const matched = descendChildNodes(found.path, found.node.children, complexTypeHandlers);
      matchedDependencies.push.apply(matchedDependencies, matched);
      continue;
    }

    if (customDescriptorHandler) {
      const typeDesc = customDescriptorHandler(found.path, found.node);
      if (typeDesc) matchedDependencies.push(typeDesc);
    }
  }

  return matchedDependencies;
}

function descendChildNodes(
  path: string,
  nodes: Array<JsonC.Node>,
  complexTypeHandlers: KeyDictionary<TJsonPackageTypeHandler>
): Array<PackageDescriptor> {
  const matchedDependencies: Array<PackageDescriptor> = [];

  for (const node of nodes) {
    if (!node.children || node.children.length === 0) continue;

    const [keyNode, valueNode] = node.children;

    // parse string properties
    if (valueNode.type == "string") {
      // create the name descriptor
      const nameDesc = createNameDescFromJsonNode(keyNode);
      // create the version descriptor
      const versionDesc = createVersionDescFromJsonNode(valueNode);
      // create the parent descriptor
      const parentDesc = createPackageParentDescType(path);
      // create the package descriptor
      const packageDesc = new PackageDescriptor([nameDesc, versionDesc, parentDesc]);

      // add the package desc to the matched array
      matchedDependencies.push(packageDesc);

      continue;
    }

    // parse complex properties
    if (valueNode.type == "object") {

      const packageDesc = new PackageDescriptor([]);

      for (const typeName in complexTypeHandlers) {

        const typeNode = JsonC.findNodeAtLocation(valueNode, [typeName]);

        if (typeNode) {
          // get the type desc
          const handler = complexTypeHandlers[typeName];

          // add the handled type to the package desc
          const typeDesc = handler(typeNode);

          // skip types that are't fully defined
          if (!typeDesc) continue;

          packageDesc.addType(typeDesc);
        }
      }

      // skip when no types were added
      if (packageDesc.typeCount === 0) continue;

      // add the name descriptor
      const nameDesc = createNameDescFromJsonNode(keyNode);
      packageDesc.addType(nameDesc)

      // add the parent path to the package desc
      const parentDesc = createPackageParentDescType(path);
      packageDesc.addType(parentDesc);

      // add the package desc to the matched array
      matchedDependencies.push(packageDesc);
    }

  }

  return matchedDependencies;
}

type FoundNode = {
  path: string,
  node: JsonC.Node | Array<JsonC.Node>
}

function findNodesAtLocation(jsonTree: JsonC.Node, expression: string): Undefinable<FoundNode> {
  const pathSegments: Array<JsonC.Segment> = expression.split(".");

  // if the path doesn't end with * then process a standard path
  if (pathSegments[pathSegments.length - 1] !== "*") {
    return {
      path: expression,
      node: JsonC.findNodeAtLocation(jsonTree, pathSegments)
    }
  }

  // find the node up until the .*
  const segmentsWithoutStar = pathSegments.slice(0, pathSegments.length - 1);
  const nodeUntilDotStar = JsonC.findNodeAtLocation(
    jsonTree,
    segmentsWithoutStar
  );

  if (!nodeUntilDotStar) return;

  if (!nodeUntilDotStar.children || nodeUntilDotStar.children.length === 0) {
    return;
  }

  // filter the childrens children where the value type is "object"
  const deepNode = nodeUntilDotStar.children
    .filter(x => x.children && x.children.length === 2)
    .flat()
    .filter(x => x.children && x.children[1].type === "object")
    .flatMap(x => x.children && x.children[1].children);

  return {
    path: segmentsWithoutStar.join("."),
    node: deepNode
  }
}