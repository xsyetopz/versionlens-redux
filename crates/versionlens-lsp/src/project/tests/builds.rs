//! The Scala version Mill and sbt builds give the `%%` and `::` artifacts they name.

use super::*;

/// Each dependency of the Mill or sbt file at `path` under `root`, as `(name, version)`.
fn build_deps(root: &Path, path: &str) -> Vec<(String, String)> {
    launched_deps(root, path, |_, _, _| None)
}

/// Each dependency of the build file at `path` under `root`, with `run` answering for commands.
fn launched_deps(root: &Path, path: &str, run: Run) -> Vec<(String, String)> {
    let path = root.join(path);
    let text = std::fs::read_to_string(&path).unwrap();
    let file = file_type_of(path.to_str().unwrap(), &text).unwrap();
    let env = Env {
        run,
        ..env(root, &[])
    };
    crate::project::extract(file, &path, &text, &env)
        .into_iter()
        .map(|d| (d.name, text[d.span].to_owned()))
        .collect()
}

#[test]
fn mill_module_takes_scala_version_from_root_build() {
    // com-lihaoyi/mill, example/large/multifile/10-multi-file-builds.
    let root = project(
        "mill-multi",
        &[
            (
                "build.mill",
                "package build\nimport mill.*, scalalib.*\n\ntrait MyModule extends ScalaModule {\n  def scalaVersion = \"3.7.4\"\n}\n",
            ),
            (
                "foo/package.mill",
                "package build.foo\nimport mill.*, scalalib.*\n\nobject `package` extends build.MyModule {\n  def moduleDeps = Seq(build.bar.qux.mymodule)\n  def mvnDeps = Seq(mvn\"com.lihaoyi::mainargs:0.7.8\")\n}\n",
            ),
            ("bar/package.mill", "package build.bar\n"),
            (
                "bar/qux/package.mill",
                "package build.bar.qux\nimport mill.*, scalalib.*\n\nobject mymodule extends build.MyModule {\n  def mvnDeps = Seq(mvn\"com.lihaoyi::scalatags:0.13.1\")\n}\n",
            ),
        ],
    );
    assert_eq!(
        build_deps(&root, "foo/package.mill"),
        [("com.lihaoyi:mainargs_3".to_owned(), "0.7.8".to_owned())]
    );
    assert_eq!(
        build_deps(&root, "bar/qux/package.mill"),
        [("com.lihaoyi:scalatags_3".to_owned(), "0.13.1".to_owned())]
    );
}

#[test]
fn mill_module_takes_named_scala_version_from_helper_file() {
    // com-lihaoyi/mill, example/large/multifile/11-helper-files, with a dependency added to foo.
    let root = project(
        "mill-helper",
        &[
            (
                "build.mill",
                "package build\nimport mill.*, scalalib.*\n\nobject `package` extends MyModule\n",
            ),
            (
                "util.mill",
                "package build\nimport mill.*, scalalib.*\n\ndef myScalaVersion = \"2.13.16\"\n\ntrait MyModule extends ScalaModule {\n  def scalaVersion = myScalaVersion\n}\n",
            ),
            (
                "foo/versions.mill",
                "package build.foo\n\ndef myProjectVersion = \"0.0.1\"\n",
            ),
            (
                "foo/package.mill",
                "package build.foo\nimport mill.*, scalalib.*\nobject `package` extends build.MyModule {\n  def mvnDeps = Seq(mvn\"org.typelevel::cats-core:2.13.0\", mvn\"org.scalamacros:::paradise:2.1.1\")\n}\n",
            ),
        ],
    );
    assert_eq!(
        build_deps(&root, "foo/package.mill"),
        [
            (
                "org.typelevel:cats-core_2.13".to_owned(),
                "2.13.0".to_owned()
            ),
            (
                "org.scalamacros:paradise_2.13.16".to_owned(),
                "2.1.1".to_owned()
            ),
        ]
    );
}

#[test]
fn mill_file_outside_build_reads_only_java_dependencies() {
    let root = project(
        "mill-alone",
        &[(
            "sub/package.mill",
            "object foo extends JavaModule {\n  def mvnDeps = Seq(mvn\"com.google.guava:guava:33.0.0-jre\", mvn\"com.lihaoyi::os-lib:0.11.4\")\n}\n",
        )],
    );
    assert_eq!(
        build_deps(&root, "sub/package.mill"),
        [("com.google.guava:guava".to_owned(), "33.0.0-jre".to_owned())]
    );
}

#[test]
fn sbt_subproject_takes_scala_version_from_root_build() {
    // Arrange: www.scala-sbt.org/1.x/docs/Multi-Project.html, a subproject's own build.sbt.
    let root = project(
        "sbt-multi",
        &[
            (
                "build.sbt",
                "ThisBuild / scalaVersion := \"2.13.16\"\nlazy val core = project\n",
            ),
            (
                "core/build.sbt",
                "libraryDependencies += \"org.typelevel\" %% \"cats-core\" % \"2.13.0\"\n",
            ),
        ],
    );

    // Act
    let found = build_deps(&root, "core/build.sbt");

    // Assert
    assert_eq!(
        found,
        [(
            "org.typelevel:cats-core_2.13".to_owned(),
            "2.13.0".to_owned()
        )]
    );
}

#[test]
fn sbt_build_without_scala_version_takes_sbt_default() {
    // Arrange: sbt 1.x is built against Scala 2.12 (Maven Central `org.scala-sbt:main_2.12`).
    let root = project(
        "sbt-default",
        &[
            ("project/build.properties", "sbt.version=1.10.7\n"),
            (
                "build.sbt",
                "libraryDependencies += \"org.typelevel\" %% \"cats-core\" % \"2.13.0\"\n",
            ),
        ],
    );

    // Act
    let found = build_deps(&root, "build.sbt");

    // Assert
    assert_eq!(
        found,
        [(
            "org.typelevel:cats-core_2.12".to_owned(),
            "2.13.0".to_owned()
        )]
    );
}

#[test]
fn sbt_reads_each_dependency_of_one_line_seq_with_launcher_default() {
    // Arrange: the fork's master fixture `parses-sbt-multiple-dependencies-on-one-line`, with no
    // `project/build.properties`, and an installed sbt 2.1.0 launcher.
    let root = project(
        "sbt-one-line",
        &[(
            "build.sbt",
            "libraryDependencies ++= Seq(\"org.zeta\" % \"zeta\" % \"1.0.0\", \"org.alpha\" %% \"alpha\" % \"2.0.0\")\n",
        )],
    );
    let run: Run = |_, program, args| {
        (program == "sbt" && args == ["--script-version"]).then(|| "2.1.0".to_owned())
    };

    // Act
    let found = launched_deps(&root, "build.sbt", run);

    // Assert
    assert_eq!(
        found,
        [
            ("org.zeta:zeta".to_owned(), "1.0.0".to_owned()),
            ("org.alpha:alpha_3".to_owned(), "2.0.0".to_owned()),
        ]
    );
}

#[test]
fn sbt_build_without_build_properties_takes_launcher_default() {
    // Arrange: the fork's master fixture `parses-sbt-double-percent-without-scala-version`, in a
    // folder with no `project/build.properties`, and an installed sbt 2.1.0 launcher.
    let root = project(
        "sbt-launcher",
        &[(
            "build.sbt",
            "libraryDependencies += \"org.typelevel\" %% \"cats-core\" % \"2.12.0\"\n",
        )],
    );
    let run: Run = |_, program, args| {
        (program == "sbt" && args == ["--script-version"]).then(|| "2.1.0".to_owned())
    };

    // Act
    let found = launched_deps(&root, "build.sbt", run);

    // Assert
    assert_eq!(
        found,
        [("org.typelevel:cats-core_3".to_owned(), "2.12.0".to_owned())]
    );
}
