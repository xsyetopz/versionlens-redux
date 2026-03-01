import type { PackageDescriptorType, PackageTypeDescriptor } from "#domain/parsers";
import type { KeyDictionary } from '#domain/utils';

/**
 * Container for multiple package-related descriptors (e.g., name, version, path).
 */
export class PackageDescriptor {

  /**
   * Initializes a new instance of the PackageDescriptor class.
   * @param descriptors Initial array of package type descriptors.
   */
  constructor(descriptors: PackageTypeDescriptor[]) {
    this.types = descriptors.length > 0
      ? Object.assign({}, ...descriptors.map(x => ({ [x.type]: x })))
      : {};

    this.typeCount = descriptors.length;
  }

  /**
   * Map of descriptors keyed by their type.
   */
  types: KeyDictionary<PackageTypeDescriptor>;

  /**
   * The number of descriptors in this container.
   */
  typeCount: number;

  /**
   * Adds a new descriptor to the container.
   * @param desc The descriptor to add.
   */
  addType(desc: PackageTypeDescriptor) {
    this.types[desc.type] = desc;
    this.typeCount++;
  }

  /**
   * Checks if a descriptor of a specific type exists in the container.
   * @param descType The type to check for.
   * @returns True if the descriptor type exists, otherwise false.
   */
  hasType(descType: keyof typeof PackageDescriptorType): boolean {
    return Reflect.has(this.types, descType);
  }

  /**
   * Gets a descriptor of a specific type.
   * @param descType The type to retrieve.
   * @returns The descriptor of the specified type.
   */
  getType<T extends PackageTypeDescriptor>(descType: keyof typeof PackageDescriptorType): T {
    return this.types[descType] as T;
  }

}