import { TPackageTypeDescriptor } from '#domain/packages';
import { AST } from 'toml-eslint-parser';

export type TTomlPackageTypeHandler = (node: AST.TOMLKeyValue) => TPackageTypeDescriptor;