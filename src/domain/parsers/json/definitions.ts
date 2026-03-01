import type { PackageDescriptor, PackageTypeDescriptor } from '#domain/parsers';
import type { KeyDictionary } from '#domain/utils';
import * as JsonC from 'jsonc-parser';

/**
 * Custom handler function for JSON descriptors.
 */
export type JsonParserCustomHandler = (path: string, valueNode: JsonC.Node) => PackageDescriptor | undefined;

/**
 * Handler function for specific package descriptor types in JSON.
 */
export type JsonPackageTypeHandler = (valueNode: JsonC.Node) => PackageTypeDescriptor;

/**
 * Options for the JSON package parser.
 */
export type JsonParserOptions = {
  /** Property names to include during parsing. */
  includePropNames: Array<string>,
  /** Optional custom descriptor handler. */
  customDescriptorHandler?: JsonParserCustomHandler,
  /** Map of handlers for complex descriptor types. */
  complexTypeHandlers: KeyDictionary<JsonPackageTypeHandler>
}

/**
 * Represents a node found at a specific path during JSON parsing.
 */
export type FoundNode = {
  /** The path to the node. */
  path: string,
  /** The node or array of nodes found. */
  node: JsonC.Node | Array<JsonC.Node> | null
}