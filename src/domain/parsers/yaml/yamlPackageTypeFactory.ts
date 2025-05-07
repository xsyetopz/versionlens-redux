import {
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptor,
  createPackageNameDesc,
  createPackageVersionDesc,
  createProjectVersionTypeDesc
} from '#domain/parsers';
import { YAMLMap } from 'yaml';

export function isNodeQuoted(node: any) {
  return node.type === "QUOTE_SINGLE"
    || node.type === "QUOTE_DOUBLE";
}

export function createNameDescFromYamlNode(keyNode: any): PackageNameDescriptor {
  const name = keyNode.value;

  const nameRange = {
    start: keyNode.range[0],
    end: keyNode.range[0],
  };

  return createPackageNameDesc(name, nameRange);
}

export function createVersionDescFromYamlNode(
  valueNode: any,
  isQuoteType: boolean
): PackageVersionDescriptor {

  const versionRange = {
    start: valueNode.range[0],
    end: valueNode.range[1]
  };

  // +1 and -1 to be inside quotes
  if (isQuoteType) {
    versionRange.start++;
    versionRange.end--;
  }

  const hasComment = valueNode.comment !== undefined && valueNode.comment.length > 0;
  const fallbackValue = hasComment ? "*" : "";
  const hasVersion = !!valueNode.value;
  const version = hasVersion ? valueNode.value : fallbackValue;
  const append = hasVersion === false && hasComment;

  return createPackageVersionDesc(
    version,
    versionRange,
    "",
    append ? " " : ""
  );
}

export function getPackageProjectVersionDesc(map: YAMLMap<any, any>): PackageDescriptor | undefined {
  for (const node of map.items) {
    if (node.key.value === 'version') {
      const isQuoted = isNodeQuoted(node.value);
      return new PackageDescriptor([
        createNameDescFromYamlNode(node.key),
        createVersionDescFromYamlNode(node.value, isQuoted),
        createProjectVersionTypeDesc()
      ]);
    }
  }
}