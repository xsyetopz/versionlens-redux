import { TPackageTypeDescriptor } from '#domain/parsers';

export type TYamlPackageTypeHandler = (
  valueNode: any,
  isQuoteType: boolean
) => TPackageTypeDescriptor;