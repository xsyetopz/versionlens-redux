import { TPackageTypeDescriptor } from '#domain/packages';
import { KeyDictionary } from '#domain/utils';
import { AST } from 'toml-eslint-parser';

export type TTomlPackageTypeHandler = (node: AST.TOMLValue) => TPackageTypeDescriptor;

export type TTomlPackageParserOptions = {
  includePropNames: Array<string>,
  complexTypeHandlers: KeyDictionary<TTomlPackageTypeHandler>
}