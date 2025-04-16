import { ILogger } from '#domain/logging';
import { createPackageResource, defaultReplaceFn, PackageDependency, TSuggestionUpdate } from '#domain/packages';
import { createPackageNameDesc, createPackageVersionDesc, createTextRange, PackageDescriptor } from '#domain/parsers';
import { ISuggestionProvider } from '#domain/providers';
import { DockerClient, DockerConfig } from '#domain/providers/docker';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { DockerfileParser } from 'dockerfile-ast';

export class DockerSuggestionProvider implements ISuggestionProvider {

  readonly name: string = 'docker';

  constructor(
    readonly client: DockerClient,
    readonly config: DockerConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("client", client);
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("logger", logger);
  }

  suggestionReplaceFn(suggestionUpdate: TSuggestionUpdate, newVersion: string): string {
    const insert = suggestionUpdate.parsedVersionPrepend.length > 0;

    return defaultReplaceFn(
      suggestionUpdate,
      // handle cases with blank version attr entries
      insert
        ? `${suggestionUpdate.parsedVersionPrepend}${newVersion}`
        : newVersion
    );
  }

  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    const re = /\n/g
    const eofPositions = [0]

    let r: RegExpExecArray
    while (r = re.exec(packageText)) eofPositions.push(r.index + 1)

    const packageDependencies = [];
    const dockerfile = DockerfileParser.parse(packageText);
    for (const from of dockerfile.getFROMs()) {
      const imageName = from.getImageName()
      const imageTag = from.getImageTag() ?? ''
      const imageNameRange = from.getImageNameRange()

      let imageTagRange = from.getImageTagRange()
      const hasTag = !!imageTagRange
      if (hasTag === false) {
        imageTagRange = {
          start: { line: imageNameRange.end.line, character: imageNameRange.end.character },
          end: { line: imageNameRange.end.line, character: imageNameRange.end.character }
        }
      }

      const nameStart = eofPositions[imageNameRange.start.line]
      const versionStart = eofPositions[imageTagRange.start.line]
      const nameRange = createTextRange(nameStart + imageNameRange.start.character, nameStart + imageNameRange.end.character)
      const versionRange = createTextRange(versionStart + imageTagRange.start.character, versionStart + imageTagRange.end.character)

      packageDependencies.push(
        new PackageDependency(
          createPackageResource(
            imageName,
            imageTag,
            packagePath
          ),
          nameRange,
          versionRange,
          new PackageDescriptor([
            createPackageNameDesc(imageName, nameRange),
            createPackageVersionDesc(imageTag, versionRange, hasTag ? '' : ':'),
          ])
        )
      );
    }

    return packageDependencies;
  }

}