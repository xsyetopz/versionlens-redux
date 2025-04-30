import { createPackageResource, PackageDependency } from '#domain/packages';
import {
  type PackageBuildDescriptor,
  type PackageImageDescriptor,
  type PackageParentDescriptor,
  type YamlParserOptions,
  createPackageNameDesc,
  createPackagePathDescType,
  createPackageRegistryDescType,
  createPackageVersionDesc,
  createTextRange,
  isNodeQuoted,
  PackageDescriptor,
  parsePackagesYaml
} from '#domain/parsers';
import { trimEndSlash } from '#domain/utils';
import { DockerfileParser } from 'dockerfile-ast';
import { isScalar } from 'yaml';

export function parseDockerCompose(
  packagePath: string,
  packageText: string,
  options: YamlParserOptions
): Array<PackageDependency> {
  const packageDependencies: Array<PackageDependency> = [];
  const parsedPackages = parsePackagesYaml(packageText, options);
  for (const descriptors of parsedPackages) {
    const parentDesc = descriptors.getType<PackageParentDescriptor>('parent')
    const imageDesc = descriptors.getType<PackageImageDescriptor>('image');
    const buildDesc = descriptors.getType<PackageBuildDescriptor>('build');

    let nameDesc
    let versionDesc
    let registryDesc
    if (imageDesc) {
      nameDesc = imageDesc.nameDesc
      versionDesc = imageDesc.versionDesc
      registryDesc = imageDesc.registry && createPackageRegistryDescType(imageDesc.registry)
    } else if (buildDesc) {
      nameDesc = createPackageNameDesc(buildDesc.pathDesc.path, buildDesc.pathDesc.pathRange)
    } else
      continue

    const descriptor = new PackageDescriptor([
      nameDesc,
      versionDesc ?? buildDesc.pathDesc,
      parentDesc,
    ]);
    if (registryDesc) descriptor.addType(registryDesc);

    packageDependencies.push(
      new PackageDependency(
        createPackageResource(
          nameDesc.name,
          versionDesc ? versionDesc.version : buildDesc.pathDesc.path,
          packagePath
        ),
        descriptor
      )
    );
  }

  return packageDependencies;
}

const eofRegEx = /\n/g
export function parseDockerfile(packagePath: string, packageText: string): Array<PackageDependency> {
  const eofPositions = [0];
  eofRegEx.lastIndex = 0;
  let r: RegExpExecArray | null;
  while (r = eofRegEx.exec(packageText)) eofPositions.push(r.index + 1);

  const packageDependencies = [];
  const dockerfile = DockerfileParser.parse(packageText);
  for (const from of dockerfile.getFROMs()) {
    const imageName = from.getImageName();
    const imageNameRange = from.getImageNameRange();
    if (imageName === null || imageNameRange === null) continue;

    const imageRegistry = from.getRegistry();
    const imageTag = from.getImageTag() ?? '';
    let imageTagRange = from.getImageTagRange()

    const hasTag = !!imageTagRange;
    if (hasTag === false) {
      imageTagRange = {
        start: { line: imageNameRange.end.line, character: imageNameRange.end.character },
        end: { line: imageNameRange.end.line, character: imageNameRange.end.character }
      };
    }

    const nameStart = eofPositions[imageNameRange.start.line];
    const versionStart = eofPositions[imageTagRange!.start.line];
    const nameRange = createTextRange(
      nameStart + imageNameRange.start.character,
      nameStart + imageNameRange.end.character
    );
    const versionRange = createTextRange(
      versionStart + imageTagRange!.start.character,
      versionStart + imageTagRange!.end.character
    );

    const descriptor = new PackageDescriptor([
      createPackageNameDesc(imageName, nameRange),
      createPackageVersionDesc(imageTag, versionRange, hasTag ? '' : ':'),
    ]);
    if (imageRegistry) descriptor.addType(createPackageRegistryDescType(imageRegistry));

    packageDependencies.push(
      new PackageDependency(
        createPackageResource(
          imageName,
          imageTag,
          packagePath
        ),
        descriptor
      )
    );
  }

  return packageDependencies;
}

const imageRegEx = /(?<registry>[^/]+[/]|)(?<image>[^:]+|)(?<tag>:.+|)/
export function createImageDescFromYamlNode(valueNode: any): PackageImageDescriptor | undefined {
  if (!valueNode.value) return

  const valueRange = {
    start: valueNode.range[0],
    end: valueNode.range[1],
  }

  if (isNodeQuoted(valueNode)) {
    valueRange.start++;
    valueRange.end--;
  }

  const match = imageRegEx.exec(valueNode.value.toString())
  if (match === null) return

  let { registry, image, tag } = match.groups;
  tag = tag.replace(':', '')

  const start = valueRange.start + registry.length;
  const imageRange = {
    start: start,
    end: start + image.length
  };
  const tagRange = tag
    ? {
      start: imageRange.end + 1,
      end: imageRange.end + tag.length + 1
    }
    : {
      start: imageRange.end,
      end: imageRange.end
    };

  return {
    type: 'image',
    nameDesc: createPackageNameDesc(image, imageRange),
    versionDesc: createPackageVersionDesc(tag ?? '', tagRange, tag ? '' : ':'),
    registry: registry ? trimEndSlash(registry) : undefined
  };
}

export function createBuildDescFromYamlNode(valueNode: any): PackageBuildDescriptor | undefined {
  if (valueNode.value === null) return

  const valueRange = {
    start: valueNode.range[0],
    end: valueNode.range[1],
  };

  if (isNodeQuoted(valueNode)) {
    valueRange.start++;
    valueRange.end--;
  }

  let dockerfile = 'dockerfile'
  if (isScalar(valueNode)) {
    return {
      type: 'build',
      pathDesc: createPackagePathDescType(`${valueNode.value}/${dockerfile}`, valueRange)
    }
  }

  const dockerfileNode = valueNode.get('dockerfile', true)
  if (dockerfileNode) dockerfile = dockerfileNode.value

  let pathDesc = undefined
  const contextNode = valueNode.get('context', true)
  if (contextNode) pathDesc = createPackagePathDescType(
    `${contextNode.value}/${dockerfile}`,
    createTextRange(contextNode.range[0], contextNode.range[1])
  )

  return { type: 'build', pathDesc }
}