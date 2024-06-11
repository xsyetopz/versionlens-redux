import { TPackageTypeDescriptor } from "#domain/packages";
import { KeyDictionary } from '#domain/utils';

export class PackageDescriptor {

  constructor(descriptors: TPackageTypeDescriptor[]) {
    this.types = descriptors.length > 0
      ? Object.assign({}, ...descriptors.map(x => ({ [x.type]: x })))
      : {};

    this.typeCount = descriptors.length;
  }

  types: KeyDictionary<TPackageTypeDescriptor>;

  typeCount: number;

  addType(desc: TPackageTypeDescriptor) {
    this.types[desc.type] = desc;
    this.typeCount++;
  }

  hasType(descType: string): boolean {
    return Reflect.has(this.types, descType);
  }

  getType<T extends TPackageTypeDescriptor>(descType: string): T {
    return this.types[descType] as T;
  }

}