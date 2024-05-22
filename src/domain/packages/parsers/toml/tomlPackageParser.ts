import { AST, parseTOML } from "toml-eslint-parser";
import { TOMLTable } from "toml-eslint-parser/lib/ast";
import { PackageDescriptorType } from "../definitions/ePackageDescriptorType";
import { PackageDescriptor } from "../packageDescriptor";
import { TTomlPackageParserOptions } from "./tTomlPackageParserOptions";
import {
  createGitDescFromTomlNode,
  createNameDescFromTomlNode,
  createPathDescFromTomlNode,
  createVersionDescFromTomlNode
} from "./tomlPackageTypeFactory";
import { complexHasProperty } from "./tomlParserUtils";

export function parsePackagesToml(
  toml: string,
  options: TTomlPackageParserOptions
): Array<PackageDescriptor> {
  try {
    const rootNode = parseTOML(toml);

    const hasChildren = rootNode.body && rootNode.body.length > 0;
    if (hasChildren === false) return [];

    return parsePackageNodes(rootNode.body[0], options);
  } catch (e) {
    return [];
  }
}

function parsePackageNodes(
  bodyNode: AST.TOMLTopLevelTable,
  options: TTomlPackageParserOptions
): Array<PackageDescriptor> {
  const matchedDependencies: Array<PackageDescriptor> = [];
  const { includePropNames } = options;

  const nodes = bodyNode.body
    .filter(x => x.type === 'TOMLTable')
    .filter((x: AST.TOMLTable) => includePropNames.includes(x.resolvedKey[0] as string))
    .map((x: AST.TOMLTable) => x.body)
    .flat()

  for (const node of nodes) {
    const parent = node.parent as TOMLTable;
    const isNameFromTable = parent.key.keys.length > 1;
    const isComplexNode = node.value.type === 'TOMLInlineTable';

    const packageDesc = isComplexNode
      ? parseComplexNode(node, node.value as AST.TOMLInlineTable)
      : parseSimpleNode(node, isNameFromTable);

    // add the package desc to the matched array
    if (packageDesc) matchedDependencies.push(packageDesc);

  }

  return matchedDependencies;
}

function parseSimpleNode(node: AST.TOMLKeyValue, isNameFromTable: boolean): PackageDescriptor {
  // add the name descriptor
  const nameDesc = createNameDescFromTomlNode(node.key, isNameFromTable);
  // add the version descriptor
  const versionDesc = createVersionDescFromTomlNode(node.value as AST.TOMLValue);

  return new PackageDescriptor([nameDesc, versionDesc]);
}

const complexTypeHandlers = {
  [PackageDescriptorType.version]: createVersionDescFromTomlNode,
  [PackageDescriptorType.path]: createPathDescFromTomlNode,
  [PackageDescriptorType.git]: createGitDescFromTomlNode
}

function parseComplexNode(nameNode: AST.TOMLKeyValue, valueNode: AST.TOMLInlineTable): PackageDescriptor {
  const packageDesc = new PackageDescriptor([]);
  for (const cNode of valueNode.body) {

    for (const typeName in complexTypeHandlers) {

      const hasType = complexHasProperty(cNode, typeName);
      if (hasType === false) continue;

      // get the type desc
      const handler = complexTypeHandlers[typeName];

      // process the type
      const typeDesc = handler(cNode.value as AST.TOMLValue);

      // add the handled type to the package desc
      packageDesc.addType(typeDesc);
      break;
    }

  }

  // skip when no types were added
  if (packageDesc.typeCount === 0) return;

  // add the name descriptor
  const nameDesc = createNameDescFromTomlNode(nameNode.key, false);
  packageDesc.addType(nameDesc)

  return packageDesc;
}