import {
  createPackageResource,
  PackageDependency
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';

export default {

  parsesRequirementsTxt: {
    test: `
# this is a comment
requests==2.25.1
flask>=2.0
django<=3.2
pytest>3.0
pkg1<1.0
pkg2~=1.0
pkg3===1.0
pkg4!=1.0
pkg5
pkg6==1.2.3 # this is a comment
`,
    expected: [
      new PackageDependency(
        createPackageResource('requests', '==2.25.1', 'test.txt'),
        new PackageDescriptor([
          createPackageNameDesc('requests', createTextRange(21)),
          createPackageVersionDesc('==2.25.1', createTextRange(29, 37))
        ])
      ),
      new PackageDependency(
        createPackageResource('flask', '>=2.0', 'test.txt'),
        new PackageDescriptor([
          createPackageNameDesc('flask', createTextRange(38)),
          createPackageVersionDesc('>=2.0', createTextRange(43, 48))
        ])
      ),
      new PackageDependency(
        createPackageResource('django', '<=3.2', 'test.txt'),
        new PackageDescriptor([
          createPackageNameDesc('django', createTextRange(49)),
          createPackageVersionDesc('<=3.2', createTextRange(55, 60))
        ])
      ),
      new PackageDependency(
        createPackageResource('pytest', '>3.0', 'test.txt'),
        new PackageDescriptor([
          createPackageNameDesc('pytest', createTextRange(61)),
          createPackageVersionDesc('>3.0', createTextRange(67, 71))
        ])
      ),
      new PackageDependency(
        createPackageResource('pkg1', '<1.0', 'test.txt'),
        new PackageDescriptor([
          createPackageNameDesc('pkg1', createTextRange(72)),
          createPackageVersionDesc('<1.0', createTextRange(76, 80))
        ])
      ),
      new PackageDependency(
        createPackageResource('pkg2', '~=1.0', 'test.txt'),
        new PackageDescriptor([
          createPackageNameDesc('pkg2', createTextRange(81)),
          createPackageVersionDesc('~=1.0', createTextRange(85, 90))
        ])
      ),
      new PackageDependency(
        createPackageResource('pkg3', '===1.0', 'test.txt'),
        new PackageDescriptor([
          createPackageNameDesc('pkg3', createTextRange(91)),
          createPackageVersionDesc('===1.0', createTextRange(95, 101))
        ])
      ),
      new PackageDependency(
        createPackageResource('pkg4', '!=1.0', 'test.txt'),
        new PackageDescriptor([
          createPackageNameDesc('pkg4', createTextRange(102)),
          createPackageVersionDesc('!=1.0', createTextRange(106, 111))
        ])
      ),
      new PackageDependency(
        createPackageResource('pkg5', '', 'test.txt'),
        new PackageDescriptor([
          createPackageNameDesc('pkg5', createTextRange(112)),
          createPackageVersionDesc('*', createTextRange(116))
        ])
      ),
      new PackageDependency(
        createPackageResource('pkg6', '==1.2.3', 'test.txt'),
        new PackageDescriptor([
          createPackageNameDesc('pkg6', createTextRange(117)),
          createPackageVersionDesc('==1.2.3', createTextRange(121, 128))
        ])
      )
    ]
  }

}
