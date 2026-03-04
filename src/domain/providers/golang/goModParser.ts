import {
  PackageDescriptor,
  createPackageGroupDesc,
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange
} from '#domain/parsers';

const INCOMPAT_BUILD = "+incompatible";
const PREPEND_V = "v";

// https://go.dev/ref/mod#go-mod-file
const DIRECTIVES = ['require', 'replace', 'exclude'];

/**
 * Parses a go.mod file content and extracts package dependencies.
 * @param text The content of the go.mod file.
 * @returns An array of Identified package descriptors.
 */
export function parsePackagesGoMod(text: string): Array<PackageDescriptor> {
  const matchedDependencies: Array<PackageDescriptor> = [];
  const lines = text.match(/[^\r\n]*(\r?\n|$)/g) || [];

  let currentBlock: string | null = null;
  let currentOffset = 0;

  for (const line of lines) {
    const trimmedLine = line.trim();

    // skip empty lines and comments at start of line
    if (trimmedLine.length === 0 || trimmedLine.startsWith('//')) {
      currentOffset += line.length;
      continue;
    }

    // check if we are entering or leaving a block
    if (trimmedLine.endsWith('(')) {
      const keyword = trimmedLine.slice(0, -1).trim();
      if (DIRECTIVES.includes(keyword)) {
        currentBlock = keyword;
      }
      currentOffset += line.length;
      continue;
    }

    if (trimmedLine === ')') {
      currentBlock = null;
      currentOffset += line.length;
      continue;
    }

    // handle single line directives or block content
    let keyword = currentBlock;
    let content = trimmedLine;

    if (!currentBlock) {
      const spaceIndex = trimmedLine.indexOf(' ');
      if (spaceIndex !== -1) {
        const potentialKeyword = trimmedLine.substring(0, spaceIndex);
        if (DIRECTIVES.includes(potentialKeyword)) {
          keyword = potentialKeyword;
          content = trimmedLine.substring(spaceIndex + 1).trim();
        }
      }
    }

    if (keyword && content.length > 0) {
      parseDirective(
        keyword,
        content,
        line,
        currentOffset,
        matchedDependencies
      );
    }

    currentOffset += line.length;
  }

  return matchedDependencies;
}

function parseDirective(
  keyword: string,
  content: string,
  line: string,
  lineOffset: number,
  matchedDependencies: Array<PackageDescriptor>
) {
  // remove inline comments
  const commentIndex = content.indexOf('//');
  const cleanContent = (commentIndex !== -1 
    ? content.substring(0, commentIndex) 
    : content).trim();

  if (cleanContent.length === 0) return;

  const parts = cleanContent.split(/\s+/);
  if (parts.length < 1) return;
  const packageName = parts[0];
  const version = parts.length > 1 ? parts[1] : '';

  // pseudo module check (v0.0.0-yyyymmddhhmmss-abcdefabcdef)
  if (version && version.split("-").length === 3) return;

  let nameStart = 0;
  if (packageName) {
    // find offsets in the original line
    const nameIndex = line.indexOf(packageName);
    if (nameIndex === -1) return;
    nameStart = lineOffset + nameIndex;
  }

  let versionStart = 0;
  let versionEnd = 0;
  if (version) {
    const versionIndex = line.indexOf(version, packageName ? (line.indexOf(packageName) + packageName.length) : 0);
    if (versionIndex !== -1) {
      versionStart = lineOffset + versionIndex;
      versionEnd = versionStart + version.length;
    }
  }

  // create the package descriptors
  const descriptors: any[] = [];
  if (packageName) {
    descriptors.push(
      createPackageNameDesc(
        packageName,
        createTextRange(nameStart, nameStart + packageName.length)
      )
    );
  }

  if (version) {
    descriptors.push(
      createPackageVersionDesc(
        version,
        createTextRange(versionStart, versionEnd),
        PREPEND_V,
        version.endsWith(INCOMPAT_BUILD) ? INCOMPAT_BUILD : ""
      )
    );
  }

  // create the group descriptor for sorting
  const groupStart = packageName ? nameStart : versionStart;
  const groupDesc = createPackageGroupDesc(
    keyword,
    createTextRange(groupStart, versionEnd || (nameStart + packageName.length))
  );
  descriptors.push(groupDesc);

  matchedDependencies.push(new PackageDescriptor(descriptors));
}
