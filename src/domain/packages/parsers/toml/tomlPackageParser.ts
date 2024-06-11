import {
  PackageDescriptor,
  PackageDescriptorType,
  TTomlPackageParserOptions,
  complexHasProperty,
  createGitDescFromTomlNode,
  createNameDescFromTomlNode,
  createPathDescFromTomlNode,
  createProjectVersionDescFromTomlNode,
  createVersionDescFromTomlNode,
  matchesTableExpression
} from "#domain/packages";
import { AST, parseTOML } from "toml-eslint-parser";

const projectVersionParentKeys = ['project', 'package'];

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

  const matchedTables = bodyNode.body
    .filter(x => x.type === 'TOMLTable')
    .map((x: AST.TOMLTable) => ({
      match: matchesTableExpression(x.resolvedKey, includePropNames),
      rows: x.body
    }))
    .filter(x => x.match);

  for (const matchedTable of matchedTables) {
    const tableRows = matchedTable.rows;
    const isPkgNameInTableName = matchedTable.match.endsWith('*');

    for (const tableRow of tableRows) {
      // check project versions
      const bareKey = tableRow.key.keys[0] as AST.TOMLBare;
      const bareParentKey = (tableRow.parent as AST.TOMLTable).key.keys[0] as AST.TOMLBare;
      if (projectVersionParentKeys.includes(bareParentKey.name) && bareKey.name === 'version') {
        matchedDependencies.push(createProjectVersionDescFromTomlNode(tableRow));
        continue;
      }

      // complex or simple
      const isComplexNode = tableRow.value.type === 'TOMLInlineTable';
      const packageDesc = isComplexNode
        ? parseComplexNode(tableRow, tableRow.value as AST.TOMLInlineTable)
        : parseSimpleNode(tableRow, isPkgNameInTableName);

      // add the package desc to the matched array
      if (packageDesc) {
        matchedDependencies.push(packageDesc);
        continue;
      }
    }
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