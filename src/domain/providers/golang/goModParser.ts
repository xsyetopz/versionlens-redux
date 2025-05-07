import {
  type PackageNameDescriptor,
  type PackageTextRange,
  type PackageVersionDescriptor,
  PackageDescriptor,
  createPackageNameDesc,
  createPackageVersionDesc
} from '#domain/parsers';

const INCOMPAT_BUILD = "+incompatible";
const PREPEND_V = "v";
const re = /(\S+(?<!retract))\s*((?=v\d+\.\d+\.\d+).[^/ \n]*)/gd

export function parsePackagesGoMod(text: string): Array<PackageDescriptor> {
  const matchedDependencies: Array<PackageDescriptor> = [];
  let match

  while ((match = re.exec(text)) !== null) {
    const packageName = match[1];
    const [packageStart] = match.indices![1];

    const version = match[2];
    const [versionStart, versionEnd] = match.indices![2];

    const skip =
      // pseudo module
      version.split("-").length === 3
      // retract [,]
      || packageName.indexOf("[") !== -1;

    if (skip) continue;

    // create the package descriptor
    const nameDesc = createGoNameDescType(packageName, packageStart);
    const versionDesc = createGoVersionDescType(
      version.trim(),
      versionStart,
      versionEnd
    );

    const packageDesc = new PackageDescriptor([nameDesc, versionDesc]);
    matchedDependencies.push(packageDesc);
  }

  return matchedDependencies;
}

function createGoNameDescType(name: string, start: number): PackageNameDescriptor {
  const nameRange: PackageTextRange = {
    start,
    end: start
  };

  return createPackageNameDesc(name, nameRange);
}

function createGoVersionDescType(version: string, start: number, end: number): PackageVersionDescriptor {
  const versionRange = {
    start,
    end
  };

  const append = version.endsWith(INCOMPAT_BUILD) ? INCOMPAT_BUILD : "";

  return createPackageVersionDesc(
    version,
    versionRange,
    PREPEND_V,
    append
  );
}