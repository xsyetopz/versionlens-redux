import { PackageDescriptor, TPackageTypeDescriptor } from '#domain/parsers';
import { KeyDictionary } from '#domain/utils';
import * as JsonC from 'jsonc-parser';

export type TJsonParserCustomHandler = (path: string, valueNode: JsonC.Node) => PackageDescriptor | undefined;

export type TJsonPackageTypeHandler = (valueNode: JsonC.Node) => TPackageTypeDescriptor;

export type TJsonPackageParserOptions = {
  includePropNames: Array<string>,
  customDescriptorHandler?: TJsonParserCustomHandler,
  complexTypeHandlers: KeyDictionary<TJsonPackageTypeHandler>
}