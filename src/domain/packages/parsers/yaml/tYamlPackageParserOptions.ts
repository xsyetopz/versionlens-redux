import { TYamlPackageTypeHandler } from "#domain/packages";
import { KeyDictionary } from '#domain/utils';

export type TYamlPackageParserOptions = {
  includePropNames: Array<string>,
  complexTypeHandlers: KeyDictionary<TYamlPackageTypeHandler>;
}