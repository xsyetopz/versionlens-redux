import { TPackageNameVersion } from '#domain/packages';

export type TPackageResource = TPackageNameVersion & {

  path: string;

};