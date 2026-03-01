import type { PackageTypeDescriptor } from '#domain/parsers';
import type { KeyDictionary } from '#domain/utils';
import { AST } from 'toml-eslint-parser';

/**
 * Handler function for specific package descriptor types in TOML.
 */
export type TomlPackageTypeHandler = (node: AST.TOMLValue) => PackageTypeDescriptor;

/**
 * Options for the TOML package parser.
 */
export type TomlParserOptions = {
  /** Table names or key paths to include during parsing. */
  includePropNames: Array<string>,
  /** Map of handlers for complex descriptor types. */
  complexTypeHandlers: KeyDictionary<TomlPackageTypeHandler>
}