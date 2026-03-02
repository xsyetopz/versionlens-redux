import {
  type TomlParserOptions,
  PackageDescriptor,
  complexHasProperty,
  createNameDescFromTomlNode,
  createProjectVersionDescFromTomlNode,
  createVersionDescFromTomlNode,
  matchesTableExpression,
  createPackageNameDesc,
  createPackageVersionDesc
} from '#domain/parsers';
import { type AST, parseTOML } from "toml-eslint-parser";

/**
 * Parses package descriptors from a TOML string.
 * @param toml The TOML content to parse.
 * @param options Parser options.
 * @returns An array of Identified package descriptors.
 */
export function parsePackagesToml(
  toml: string,
  options: TomlParserOptions
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

/**
 * Parses package nodes from a TOML table.
 * @param bodyNode The top-level TOML table node.
 * @param options Parser options.
 * @returns An array of Identified package descriptors.
 */
function parsePackageNodes(
  bodyNode: AST.TOMLTopLevelTable,
  options: TomlParserOptions
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
        continue;
      }

      const isArrayNode = tableRow.value.type === 'TOMLArray';
      const isDependenciesKey = rowKey.name == 'dependencies' && isArrayNode;
      const isOptionalDependenciesRow = matchedTable.match == 'project.optional-dependencies' && isArrayNode;

      if (isDependenciesKey || isOptionalDependenciesRow) {
        matchedDependencies.push(...parseArrayNode(tableRow.value as AST.TOMLArray));
        continue;
      }

      // ignore other [package] table rows
      if (isPackageRow) continue;

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

/**
 * Parses a simple key-value pair in TOML.
 * @param node The TOML key-value node.
 * @param isNameFromTable Whether the package name should be derived from the table name.
 * @returns A package descriptor.
 */
function parseSimpleNode(node: AST.TOMLKeyValue, isNameFromTable: boolean): PackageDescriptor {
  // add the name descriptor
  const nameDesc = createNameDescFromTomlNode(node.key, isNameFromTable);
  // add the version descriptor
  const versionDesc = createVersionDescFromTomlNode(node.value as AST.TOMLValue);

  return new PackageDescriptor([nameDesc, versionDesc]);
}

/**
 * Parses a complex inline table node in TOML.
 * @param nameNode The TOML key-value node containing the name.
 * @param valueNode The TOML inline table node.
 * @param options Parser options.
 * @returns A package descriptor or undefined.
 */
function parseComplexNode(
  nameNode: AST.TOMLKeyValue,
  valueNode: AST.TOMLInlineTable,
  options: TomlParserOptions
): PackageDescriptor | undefined {
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

/**
 * Parses a TOML array containing PEP 508 dependency strings.
 * @param arrayNode The TOML array node.
 * @returns An array of package descriptors.
 */
function parseArrayNode(
  arrayNode: AST.TOMLArray
): Array<PackageDescriptor> {
  const descriptors: Array<PackageDescriptor> = [];

  for (const element of arrayNode.elements) {
    if (element.type !== 'TOMLValue' || typeof element.value !== 'string') continue;

    const rawText = element.value;
    const startOffset = element.range[0] + 1;

    const markerIndex = rawText.indexOf(';');
    const packageSpec = markerIndex === -1 ? rawText : rawText.substring(0, markerIndex);

    const match = /^([a-zA-Z0-9_\-\.]+)(?:\[[^\]]*\])?(.*)$/.exec(packageSpec);
    if (!match) continue;

    const name = match[1];
    const version = match[2].trim();

    const nameStart = startOffset + packageSpec.indexOf(name);
    const nameEnd = nameStart + name.length;
    const nameRange = { start: nameStart, end: nameEnd };

    const nameDesc = createPackageNameDesc(name, nameRange);

    let versionDesc;
    if (version) {
      const versionStart = startOffset + packageSpec.indexOf(version, name.length);
      const versionEnd = versionStart + version.length;
      const versionRange = { start: versionStart, end: versionEnd };
      versionDesc = createPackageVersionDesc(version, versionRange);
    } else {
      const versionRange = { start: startOffset + packageSpec.length, end: startOffset + packageSpec.length };
      versionDesc = createPackageVersionDesc("", versionRange);
    }

    descriptors.push(new PackageDescriptor([nameDesc, versionDesc]));
  }

  return descriptors;
}