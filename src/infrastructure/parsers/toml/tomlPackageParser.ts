import { PackageDescriptor } from '#domain/packages';
import {
  TTomlPackageParserOptions,
  complexHasProperty,
  createNameDescFromTomlNode,
  createProjectVersionDescFromTomlNode,
  createVersionDescFromTomlNode,
  matchesTableExpression
} from '#infrastructure/parsers';
import { AST, parseTOML } from "toml-eslint-parser";

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
    .map(
      (x: AST.TOMLTable) => ({
        match: matchesTableExpression(x.resolvedKey, includePropNames),
        name: (x.key.keys[0] as AST.TOMLBare).name,
        rows: x.body
      })
    )
    .filter(x => x.match);

  for (const matchedTable of matchedTables) {
    const tableRows = matchedTable.rows;
    const isPkgNameInTableName = matchedTable.match.endsWith('*');

    for (const tableRow of tableRows) {
      const rowKey = tableRow.key.keys[0] as AST.TOMLBare;
      const isPackageRow = matchedTable.name === "package" || matchedTable.name === "project";

      // add version desc for [package] tables
      if (isPackageRow && rowKey.name === 'version') {
        matchedDependencies.push(createProjectVersionDescFromTomlNode(tableRow));
        break;
      }

      // ignore other [package] table rows
      if (isPackageRow) break;

      // complex or simple
      const isComplexNode = tableRow.value.type === 'TOMLInlineTable';
      const packageDesc = isComplexNode
        ? parseComplexNode(tableRow, tableRow.value as AST.TOMLInlineTable, options)
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

function parseComplexNode(
  nameNode: AST.TOMLKeyValue,
  valueNode: AST.TOMLInlineTable,
  options: TTomlPackageParserOptions
): PackageDescriptor {
  const packageDesc = new PackageDescriptor([]);
  const complexTypeHandlers = options.complexTypeHandlers;

  for (const node of valueNode.body) {

    for (const typeName in complexTypeHandlers) {

      const hasType = complexHasProperty(node, typeName);
      if (hasType === false) continue;

      // get the type desc
      const handler = complexTypeHandlers[typeName];

      // process the type
      const typeDesc = handler(node.value as AST.TOMLValue);

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