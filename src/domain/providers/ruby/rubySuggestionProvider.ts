import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  ClientResponseFactory,
  PackageVersionType,
  VersionUtils,
  type PackageClientRequest,
  type PackageClientResponse,
  type PackageDependency,
  type SuggestionUpdate
} from '#domain/packages';
import {
  PackageDescriptorType,
  type PackageGitHubDescriptor,
  type PackagePathDescriptor
} from '#domain/parsers';
import type { ISuggestionProvider } from '#domain/providers';
import { RubyConfig, RubySuggestionResolver, parseGemfile, rubyReplaceVersion } from '#domain/providers/ruby';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Regex to extract the operator from a version string.
 */
const operatorRegex = /^(~>|>=|>|<=|<|==|!=)\s*/;

/**
 * Provides suggestions for Ruby dependencies by parsing Gemfiles and resolving package versions.
 */
export class RubySuggestionProvider implements ISuggestionProvider {

  /**
   * The name of the suggestion provider.
   */
  readonly name: string = 'ruby';

  /**
   * Initializes a new instance of the RubySuggestionProvider class.
   * @param resolver The resolver used to fetch suggestions.
   * @param config The configuration for the Ruby provider.
   * @param logger The logger for this provider.
   */
  constructor(
    readonly resolver: RubySuggestionResolver,
    readonly config: RubyConfig,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('resolver', resolver);
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Handles the replacement of Ruby version constraints.
   * @param suggestion The suggestion being applied.
   * @param _newVersion The new version string.
   * @returns The updated version string with operators.
   */
  suggestionReplaceFn(suggestion: SuggestionUpdate, _newVersion: string): string {
    return rubyReplaceVersion(suggestion, operatorRegex);
  }

  /**
   * Parses dependencies from a Gemfile.
   * @param packagePath The path to the package file.
   * @param packageText The content of the package file.
   * @returns An array of identified package dependencies.
   */
  parseDependencies(packagePath: string, packageText: string): Array<PackageDependency> {
    return parseGemfile(packagePath, packageText);
  }

  /**
   * Fetches suggestions for a given package request.
   * @param request The package client request.
   * @returns A promise resolving to the package client response containing suggestions.
   */
  async fetchSuggestions(request: PackageClientRequest<null>): Promise<PackageClientResponse> {
    const { descriptors, package: requestedPackage } = request.parsedDependency;

    if (descriptors.hasType(PackageDescriptorType.path)) {
      const pathDesc = descriptors.getType<PackagePathDescriptor>(
        PackageDescriptorType.path
      );
      return await this.resolver.fromPath(
        requestedPackage.name,
        requestedPackage.path,
        pathDesc.path
      );
    }

    if (descriptors.hasType(PackageDescriptorType.github)) {
      const githubDesc = descriptors.getType<PackageGitHubDescriptor>(
        PackageDescriptorType.github
      );

      // detect if we are using a tag
      const isTag = requestedPackage.version.includes('tag:');

      return await this.resolver.fromGitHub(
        githubDesc.githubUrl, 
        githubDesc.githubRef || '',
        isTag
      );
    }

    if (descriptors.hasType(PackageDescriptorType.git)) {
      return this.resolver.fromGit();
    }
    
    // Normalize ~> to ~ for semver parsing if needed
    const versionToParse = requestedPackage.version.replace('~>', '~');
    
    const semverSpec = VersionUtils.parseSemver(versionToParse);
    if (semverSpec === null) {
      return ClientResponseFactory.createInvalidVersion(
        ClientResponseFactory.createResponseStatus(ClientResponseSource.local, 400),
        PackageVersionType.Version
      );
    }
    
    // Restore the original version for the resolver
    semverSpec.rawVersion = requestedPackage.version;

    return await this.resolver.resolve(request, semverSpec);
  }

}
