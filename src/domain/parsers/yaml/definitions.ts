import { PackageTypeDescriptor } from '#domain/parsers';
import { KeyDictionary } from '#domain/utils';

/**
 * Options for the YAML package parser.
 */
export type YamlParserOptions = {
  /** Property names to include during parsing. */
  includePropNames: Array<string>,
  /** Map of handlers for complex descriptor types. */
  complexTypeHandlers: KeyDictionary<YamlTypeDescriptorHandler>;
}

/**
 * Handler function for specific package descriptor types in YAML.
 */
export type YamlTypeDescriptorHandler = (
  valueNode: any,
  isQuoteType: boolean
) => PackageTypeDescriptor | undefined;
