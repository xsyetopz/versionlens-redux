//! Legacy manifests: Maven 1, Ivy overrides, Buildr, `.nuspec`, PEAR and PECL, CPAN metadata,
//! `Dist::Zilla`, and Component. Fixtures are excerpts of the real files each row cites.

use super::expect;
use crate::catalog::file_type_of;

#[test]
fn reads_maven1_project_xml() {
    let text = "<project>\n  <pomVersion>3</pomVersion>\n  <dependencies>\n    <dependency>\n      <id>junit</id>\n      <version>3.7</version>\n    </dependency>\n    <dependency>\n      <groupId>commons-logging</groupId>\n      <artifactId>commons-logging-api</artifactId>\n      <version>1.0.4</version>\n    </dependency>\n  </dependencies>\n</project>\n";
    expect(
        "commons-lang/project.xml",
        text,
        &[
            ("junit:junit", "3.7", "maven"),
            ("commons-logging:commons-logging-api", "1.0.4", "maven"),
        ],
    );
}

#[test]
fn reads_ivy_overrides() {
    let text = "<ivy-module version=\"2.0\">\n  <dependencies>\n    <dependency org=\"commons-lang\" name=\"commons-lang\" rev=\"2.0\"/>\n    <override org=\"org.slf4j\" module=\"slf4j-api\" rev=\"1.7.36\"/>\n  </dependencies>\n</ivy-module>\n";
    expect(
        "ivy.xml",
        text,
        &[
            ("commons-lang:commons-lang", "2.0", "maven"),
            ("org.slf4j:slf4j-api", "1.7.36", "maven"),
        ],
    );
}

#[test]
fn reads_buildr_artifact_specs() {
    let text = "JUNIT = 'junit:junit:jar:4.10'\ndefine 'iron_mq' do\n  compile.with 'com.google.code.gson:gson:jar:2.1',\n               \"org.lwjgl:lwjgl:jar:natives-linux:3.3.1\"\n  package :jar\nend\n";
    expect(
        "buildfile",
        text,
        &[
            ("junit:junit", "4.10", "maven"),
            ("com.google.code.gson:gson", "2.1", "maven"),
            ("org.lwjgl:lwjgl", "3.3.1", "maven"),
        ],
    );
}

#[test]
fn reads_nuspec_dependencies_and_groups() {
    let flat = "<?xml version=\"1.0\"?>\n<package xmlns=\"http://schemas.microsoft.com/packaging/2013/05/nuspec.xsd\">\n  <metadata>\n    <id>Sample</id>\n    <dependencies>\n      <dependency id=\"PackageA\" version=\"1.1.0\" />\n      <dependency id=\"PackageB\" version=\"[1,2)\" />\n    </dependencies>\n  </metadata>\n</package>\n";
    expect(
        "Sample.nuspec",
        flat,
        &[
            ("PackageA", "1.1.0", "nuget"),
            ("PackageB", "[1,2)", "nuget"),
        ],
    );
    let grouped = "<package>\n  <metadata>\n    <dependencies>\n      <group>\n        <dependency id=\"RouteMagic\" version=\"1.1.0\" />\n      </group>\n      <group targetFramework=\"net40\">\n        <dependency id=\"jQuery\" version=\"1.6.2\" />\n      </group>\n    </dependencies>\n  </metadata>\n</package>\n";
    expect(
        "Sample.nuspec",
        grouped,
        &[
            ("RouteMagic", "1.1.0", "nuget"),
            ("jQuery", "1.6.2", "nuget"),
        ],
    );
}

#[test]
fn reads_pear_packages_and_pecl_extensions() {
    let text = "<package version=\"2.0\" xmlns=\"http://pear.php.net/dtd/package-2.0\">\n <dependencies>\n  <required>\n   <php>\n    <min>5.2.0</min>\n   </php>\n   <package>\n    <name>Console_Getopt</name>\n    <channel>pear.php.net</channel>\n    <min>1.4.1</min>\n   </package>\n  </required>\n  <optional>\n   <package>\n    <name>Console_Color2</name>\n    <channel>pear.php.net</channel>\n    <min>0.1.2</min>\n   </package>\n   <extension>\n    <name>igbinary</name>\n    <min>2.0</min>\n   </extension>\n  </optional>\n </dependencies>\n</package>\n";
    expect(
        "package.xml",
        text,
        &[
            ("Console_Getopt", "1.4.1", "pear"),
            ("Console_Color2", "0.1.2", "pear"),
            ("igbinary", "2.0", "pecl"),
        ],
    );
    // A ROS package.xml shares the name and has no PEAR dependency tree.
    let ros =
        "<package format=\"2\">\n  <name>demo</name>\n  <depend>roscpp</depend>\n</package>\n";
    expect("ros_ws/src/demo/package.xml", ros, &[]);
}

#[test]
fn reads_cpan_metadata() {
    let json = r#"{"prereqs": {"configure": {"requires": {"Dist::CheckConflicts": "0.02", "perl": "5.008003"}, "suggests": {"JSON::PP": "2.27300"}}, "runtime": {"recommends": {"Data::OptList": "0.107"}}}}"#;
    expect(
        "META.json",
        json,
        &[
            ("Dist::CheckConflicts", "0.02", "cpan"),
            ("perl", "5.008003", "cpan"),
            ("JSON::PP", "2.27300", "cpan"),
            ("Data::OptList", "0.107", "cpan"),
        ],
    );
    let yml = "name: libwww-perl\nconfigure_requires:\n    ExtUtils::MakeMaker:  0\nrequires:\n    HTML::Parser:         3.33\n    URI:                  1.10\nresources:\n    repository:   http://github.com/gisle/libwww-perl\n";
    expect(
        "META.yml",
        yml,
        &[
            ("ExtUtils::MakeMaker", "0", "cpan"),
            ("HTML::Parser", "3.33", "cpan"),
            ("URI", "1.10", "cpan"),
        ],
    );
}

#[test]
fn reads_dist_zilla_prereqs() {
    let text = "name    = Dist-Zilla\n\n[@RJBS]\nperl-window   = standard\n\n[Prereqs / TestRequires]\nTest::More = 0.90\n\n[Prereqs / DevelopNeeds]\n-phase = develop\n-relationship = requires\nTest::Pod = 1.41\n\n[Prereqs]\nperl    = v5.22.0\nparent  = 0 ; used by the AutoPrereq test corpus\nConfig::MVP              = 2.200011 ; add cwd to lib during config reading\n\n[OSPrereqs / MSWin32]\nDateTime::TimeZone = 1.92\n\n[Conflicts]\nCatalyst = 5.90049999\n\n[Breaks]\nDist::Zilla::Plugin::Run = <= 0.035\n";
    expect(
        "dist.ini",
        text,
        &[
            ("Test::More", "0.90", "cpan"),
            ("Test::Pod", "1.41", "cpan"),
            ("perl", "v5.22.0", "cpan"),
            ("parent", "0", "cpan"),
            ("Config::MVP", "2.200011", "cpan"),
            ("DateTime::TimeZone", "1.92", "cpan"),
        ],
    );
}

#[test]
fn reads_component_json() {
    let text = r#"{"name": "app", "dependencies": {"component/emitter": "*", "visionmedia/page.js": "1.3.0"}, "development": {"dependencies": {"component/assert": "0.4.0"}}}"#;
    let file = file_type_of("component.json", text).map(|f| f.id);
    assert_eq!(file, Some("component"));
    expect(
        "component.json",
        text,
        &[
            ("component/emitter", "*", "github"),
            ("visionmedia/page.js", "1.3.0", "github"),
            ("component/assert", "0.4.0", "github"),
        ],
    );
}
