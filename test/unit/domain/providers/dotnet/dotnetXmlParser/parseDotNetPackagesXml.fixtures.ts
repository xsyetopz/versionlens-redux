import {
  createDependencyRange,
  createPackageNameDesc,
  createPackageVersionDesc,
  createProjectVersionTypeDesc,
  PackageDescriptor
} from '#domain/parsers';

export default {

  parsesItemGroupPackages: {

    test: `
      <Project>
        <Sdk Name="Microsoft.Build.CentralPackageVersions" Version="2.1.3" />
        <ItemGroup>
            <PackageReference Include="Microsoft.Extensions.DependencyInjection.Abstractions" Version="2.0.0" />
            <PackageReference Include="Microsoft.Extensions.Logging.Abstractions" Version="2.0.1" />
            <PackageVersion Include="System.Text.Json" Version="4.7.2" />
            <PackageVersion Include="Microsoft.Extensions.Options" VersionOverride="1.2.3" />
            <DotNetCliToolReference Include="Microsoft.EntityFrameworkCore.Tools" Version="6.0.7" />
            <GlobalPackageReference Include="Microsoft.Azure.ServiceBus" Version="(3.0,)" />
            <PackageVersion Update="AngularJS.Core" Version="1.0.*" />
            <PackageReference Include="NoVersionAttribute" />
            <PackageReference Include="ChildVersionNoAttribute">
              <!-- should ignore -->
              <Version></Version>
            </PackageReference>
        </ItemGroup>
      </Project>
    `,

    expected: [
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.Build.CentralPackageVersions",
          createDependencyRange(25, 25)
        ),
        createPackageVersionDesc("2.1.3", createDependencyRange(85, 90))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.Azure.ServiceBus",
          createDependencyRange(610, 610)
        ),
        createPackageVersionDesc("(3.0,)", createDependencyRange(680, 686))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.Extensions.DependencyInjection.Abstractions",
          createDependencyRange(127, 127)
        ),
        createPackageVersionDesc("2.0.0", createDependencyRange(218, 223))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.Extensions.Logging.Abstractions",
          createDependencyRange(240, 240)
        ),
        createPackageVersionDesc("2.0.1", createDependencyRange(319, 324))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "NoVersionAttribute",
          createDependencyRange(774, 774)
        ),
        createPackageVersionDesc("*", createDependencyRange(821, 821), 'Version="', '" ')
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "ChildVersionNoAttribute",
          createDependencyRange(836, 836)
        ),
        createPackageVersionDesc("*", createDependencyRange(887, 887), ' Version="', '"')
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "System.Text.Json",
          createDependencyRange(341, 341)
        ),
        createPackageVersionDesc("4.7.2", createDependencyRange(393, 398))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.Extensions.Options",
          createDependencyRange(415, 415)
        ),
        createPackageVersionDesc("1.2.3", createDependencyRange(487, 492))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "AngularJS.Core",
          createDependencyRange(703, 703)
        ),
        createPackageVersionDesc("1.0.*", createDependencyRange(752, 757))
      ]),
      new PackageDescriptor([
        createPackageNameDesc(
          "Microsoft.EntityFrameworkCore.Tools",
          createDependencyRange(509, 509)
        ),
        createPackageVersionDesc("6.0.7", createDependencyRange(588, 593))
      ]),
    ]
  },

  parsesPropertyGroupVersions: {
    test: `
      <Project>
        <PropertyGroup>
            <AssemblyVersion>1.2.3</AssemblyVersion>
            <Version></Version>
        </PropertyGroup>
      </Project>
    `,
    expected: [
      new PackageDescriptor([
        createPackageNameDesc("AssemblyVersion", createDependencyRange(53, 53)),
        createPackageVersionDesc("1.2.3", createDependencyRange(70, 75)),
        createProjectVersionTypeDesc()
      ]),
      new PackageDescriptor([
        createPackageNameDesc("Version", createDependencyRange(106, 106)),
        createPackageVersionDesc("", createDependencyRange(115, 115)),
        createProjectVersionTypeDesc()
      ]),
    ]
  }
}