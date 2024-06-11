import { PackageVersionType } from "#domain/packages";

export type TSemverSpec = {

  rawVersion: string,

  type: PackageVersionType,

};