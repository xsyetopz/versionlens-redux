import { PackageDescriptor, TPackageTypeDescriptor } from '#domain/packages';
import { KeyDictionary } from 'domain/utils';
import * as JsonC from 'jsonc-parser';

export type TJsonParserCustomHandler = (path: string, valueNode: JsonC.Node) => PackageDescriptor;

export type TJsonPackageTypeHandler = (valueNode: JsonC.Node) => TPackageTypeDescriptor;

export type TJsonPackageParserOptions = {
  includePropNames: Array<string>,
  customDescriptorHandler?: TJsonParserCustomHandler,
  complexTypeHandlers: KeyDictionary<TJsonPackageTypeHandler>
}