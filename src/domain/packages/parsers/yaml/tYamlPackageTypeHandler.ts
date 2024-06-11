import { TPackageTypeDescriptor } from '#domain/packages';

export type TYamlPackageTypeHandler = (
  valueNode: any,
  isQuoteType: boolean
) => TPackageTypeDescriptor;