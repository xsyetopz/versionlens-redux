import {
  createPackageManifest,
  PackageDependency
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createPackagePathDescType,
  createPackageGitDescType,
  createPackageGitHubDescType,
  createPackageGroupDesc,
  createTextRange,
  PackageDescriptor,
  type PackageTypeDescriptor
} from '#domain/parsers';

/**
 * Regex to match gem name in Gemfile.
 * Group 1: Quote char for name
 * Group 2: Package name
 */
const gemNameRegex = /^\s*gem\s+(['"])(?<name>[^'"]+)\1/;

/**
 * Regexes for gem options
 */
const versionRegex = /,\s*(['"])(?<version>[^'"]*)\1/;
const pathRegex = /,\s*path:\s*(['"])(?<path>[^'"]+)\1/;
const gitRegex = /,\s*git:\s*(['"])(?<git>[^'"]+)\1/;
const githubRegex = /,\s*github:\s*(['"])(?<github>[^'"]+)\1/;
const refRegex = /,\s*ref:\s*(['"])(?<ref>[^'"]*)\1/;
const branchRegex = /,\s*branch:\s*(['"])(?<branch>[^'"]*)\1/;
const tagRegex = /,\s*tag:\s*(['"])(?<tag>[^'"]*)\1/;

/**
 * Regex for group start
 */
const groupStartRegex = /^\s*group\s+(?:(?::\w+)|(?:['"]\w+['"])|(?:\[(?::\w+,\s*)*(?::\w+)\]))(?:\s*,\s*(?:(?::\w+)|(?:['"]\w+['"])|(?:\[(?::\w+,\s*)*(?::\w+)\])))*\s+do/;

/**
 * Regex for group end
 */
const groupEndRegex = /^\s*end\s*$/;

/**
 * Parses a Gemfile to identify package dependencies.
 * @param packagePath The path to the Gemfile.
 * @param packageText The content of the file.
 * @returns An array of identified package dependencies.
 */
export function parseGemfile(
  packagePath: string,
  packageText: string
): Array<PackageDependency> {
  const dependencies: Array<PackageDependency> = [];
  let currentOffset = 0;
  const groupStack: string[] = ['dependencies'];

  const lines = packageText.match(/[^\r\n]*(\r?\n|$)/g) || [];

  for (const line of lines) {
    const trimmedLine = line.trim();

    if (trimmedLine.length > 0 && !trimmedLine.startsWith('#')) {
      
      // Check for group end
      if (groupEndRegex.test(line)) {
        if (groupStack.length > 1) {
          groupStack.pop();
        }
      }

      // Check for group start
      const groupMatch = groupStartRegex.exec(line);
      if (groupMatch) {
        // extract group names from the match (simple version for now)
        const groupName = groupMatch[0].trim().replace(/\s+do$/, '');
        groupStack.push(groupName);
      }

      const nameMatch = gemNameRegex.exec(line);
      if (nameMatch) {
        const groups = nameMatch.groups!;
        const packageName = groups.name;

        // Calculate name range (at the start of the 'gem' for CodeLens positioning)
        const nameStart = currentOffset + line.indexOf('gem');

        const descriptors: Array<PackageTypeDescriptor> = [
          createPackageNameDesc(packageName, createTextRange(nameStart))
        ];

        let manifestVersion = '';

        // Parse options
        const pathMatch = pathRegex.exec(line);
        const gitMatch = gitRegex.exec(line);
        const githubMatch = githubRegex.exec(line);
        const versionMatch = versionRegex.exec(line);
        
        const tagMatch = tagRegex.exec(line);
        const branchMatch = branchRegex.exec(line);
        const refMatchOnly = refRegex.exec(line);
        const refMatch = tagMatch || branchMatch || refMatchOnly;

        if (githubMatch || gitMatch) {
          const isGithub = !!githubMatch;
          const match = isGithub ? githubMatch : gitMatch;
          const url = isGithub ? match!.groups!.github : match!.groups!.git;
          const githubUrl = isGithub ? `https://github.com/${url}.git` : url;
          
          let gitRef = '';
          if (tagMatch) {
            gitRef = tagMatch.groups!.tag;
          } else if (branchMatch) {
            gitRef = branchMatch.groups!.branch;
          } else if (refMatchOnly) {
            gitRef = refMatchOnly.groups!.ref;
          }
          
          const optionName = isGithub ? 'github:' : 'git:';
          const startInLine = line.indexOf(optionName);
          let endInLine = line.indexOf(match![0]) + match![0].length;
          if (refMatch) {
            endInLine = Math.max(endInLine, line.indexOf(refMatch[0]) + refMatch[0].length);
          }
          
          const gitRange = createTextRange(
            currentOffset + startInLine,
            currentOffset + endInLine
          );

          if (isGithub) {
            descriptors.push(createPackageGitHubDescType(githubUrl, gitRef, gitRange));
          } else {
            descriptors.push(createPackageGitDescType(githubUrl, '', gitRef, gitRange));
          }

          manifestVersion = line.substring(startInLine, endInLine);
        } else if (pathMatch) {
          const rawPath = pathMatch.groups!.path;
          const pathStartInLine = line.indexOf(rawPath, line.indexOf('path:'));
          const pathRange = createTextRange(
            currentOffset + pathStartInLine,
            currentOffset + pathStartInLine + rawPath.length
          );
          descriptors.push(createPackagePathDescType(rawPath, pathRange));
          manifestVersion = rawPath;
        } else if (versionMatch) {
          const rawVersion = versionMatch.groups!.version;
          const versionStartInLine = line.indexOf(rawVersion, line.indexOf(packageName) + packageName.length);
          const versionRange = createTextRange(
            currentOffset + versionStartInLine,
            currentOffset + versionStartInLine + rawVersion.length
          );
          descriptors.push(createPackageVersionDesc(rawVersion, versionRange));
          manifestVersion = rawVersion;
        } else {
          // Special case for blank versions: assume latest
          const nameEndInLine = line.indexOf(packageName) + packageName.length + 1;
          const versionRange = createTextRange(currentOffset + nameEndInLine);
          descriptors.push(createPackageVersionDesc('*', versionRange, ", '", "'"));
          manifestVersion = '';
        }

        const lineWithoutNewline = line.replace(/(\r?\n)$/, '');
        descriptors.push(
          createPackageGroupDesc(
            groupStack[groupStack.length - 1],
            createTextRange(currentOffset, currentOffset + lineWithoutNewline.length)
          )
        );

        dependencies.push(
          new PackageDependency(
            createPackageManifest(
              packageName,
              manifestVersion,
              packagePath
            ),
            new PackageDescriptor(descriptors)
          )
        );
      }
    }

    currentOffset += line.length;
  }

  return dependencies;
}
