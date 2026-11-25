//! Build systems: CPM.cmake, `FetchContent`, Hunter, `PlatformIO`, ESP-IDF components, Mill, fpm, Boss,
//! Buck2, Pants, west, Alire, Arduino, and Nimble.
//! Fixtures are excerpts of the real files or docs each row cites.

use super::expect;

#[test]
fn reads_cpm_shorthand_and_named_packages() {
    // TheLartians/ModernCppStarter and the CPM.cmake README and examples.
    let text = "cmake_minimum_required(VERSION 3.14 FATAL_ERROR)\nproject(Greeter VERSION 1.0 LANGUAGES CXX)\nCPMAddPackage(\"gh:TheLartians/PackageProject.cmake@1.13.0\")\nCPMAddPackage(\"gh:fmtlib/fmt#7.1.3\")\nCPMAddPackage(\"gh:jbeder/yaml-cpp#yaml-cpp-0.6.3@0.6.3\")\nCPMAddPackage(\n  NAME fmt\n  GIT_TAG 12.1.0\n  GITHUB_REPOSITORY fmtlib/fmt\n  OPTIONS \"FMT_INSTALL YES\" # create an installable target\n)\nCPMAddPackage(\n  NAME benchmark\n  GITHUB_REPOSITORY google/benchmark\n  VERSION 1.7.1\n  OPTIONS \"BENCHMARK_ENABLE_TESTING Off\"\n)\nCPMAddPackage(NAME Greeter SOURCE_DIR ${CMAKE_CURRENT_LIST_DIR}/..)\n";
    expect(
        "CMakeLists.txt",
        text,
        &[
            ("TheLartians/PackageProject.cmake", "1.13.0", "github"),
            ("fmtlib/fmt", "7.1.3", "github"),
            ("jbeder/yaml-cpp", "0.6.3", "github"),
            ("fmtlib/fmt", "12.1.0", "github"),
            ("google/benchmark", "1.7.1", "github"),
        ],
    );
}

#[test]
fn reads_fetchcontent_repositories_and_archives() {
    let text = "FetchContent_Declare(\n  json\n  GIT_REPOSITORY https://github.com/nlohmann/json.git\n  GIT_TAG v3.11.3\n)\nCPMAddPackage(\n  NAME lua\n  GIT_REPOSITORY https://github.com/lua/lua.git\n  VERSION 5.3.5\n  DOWNLOAD_ONLY YES\n)\nCPMAddPackage(\n  NAME     spdlog\n  URL      https://github.com/gabime/spdlog/archive/refs/tags/v1.12.0.zip\n)\n";
    expect(
        "cmake/deps.cmake",
        text,
        &[
            ("nlohmann/json", "v3.11.3", "github"),
            ("lua/lua", "5.3.5", "github"),
            ("gabime/spdlog", "v1.12.0", "github"),
        ],
    );
}

#[test]
fn reads_hunter_config_versions() {
    let text = "hunter_config(\n    Boost\n    # Version from \"project/${package}/hunter.cmake\"\n    VERSION 1.72.0-p0\n    CMAKE_ARGS OPTION1=OFF\n)\nhunter_config(GTest VERSION 1.10.0)\nhunter_config(foo GIT_SELF)\n";
    expect(
        "cmake/Hunter/config.cmake",
        text,
        &[
            ("Boost", "1.72.0-p0", "hunter"),
            ("GTest", "1.10.0", "hunter"),
        ],
    );
}

#[test]
fn reads_platformio_platforms_tools_and_libraries() {
    // esphome/esphome's platformio.ini, with the lib_deps forms of the PlatformIO docs.
    let text = "[common:esp8266-arduino]\nplatform = platformio/espressif8266@4.2.1\nplatform_packages =\n    platformio/framework-arduinoespressif8266@~3.30102.0\n    pioarduino/framework-espidf@https://github.com/pioarduino/esp-idf/releases/download/v6.1.1/esp-idf-v6.1.1.tar.xz\nlib_deps =\n    ${common:arduino.lib_deps}\n    ; owner-based declaration\n    knolleary/PubSubClient\n    fastled/FastLED@3.9.16                ; fastled_base\n    ESP8266WiFi                           ; wifi (Arduino built-in)\n    bblanchon/ArduinoJson @ ~5.6,!=5.4\n    adafruit/Adafruit GFX Library @ ^1.11.9\n\n[env:exact]\nplatform = espressif32 @ 6.5.0\nlib_deps = google/googletest@^1.15.2\n";
    expect(
        "platformio.ini",
        text,
        &[
            ("espressif8266", "4.2.1", "platformio-platform"),
            (
                "platformio/framework-arduinoespressif8266",
                "~3.30102.0",
                "platformio-tool",
            ),
            ("fastled/FastLED", "3.9.16", "platformio"),
            ("bblanchon/ArduinoJson", "~5.6,!=5.4", "platformio"),
            ("adafruit/Adafruit GFX Library", "^1.11.9", "platformio"),
            ("espressif32", "6.5.0", "platformio-platform"),
            ("google/googletest", "^1.15.2", "platformio"),
        ],
    );
}

#[test]
fn reads_idf_component_dependencies() {
    // espressif/esp-rainmaker's examples/switch, with the short form and a default namespace.
    let text = "dependencies:\n  ## Required IDF version\n  idf:\n    version: \">=5.0.0\"\n  espressif/button:\n    version: \"^4.1.4\"\n  lvgl/lvgl: \"9.6.0\"\n  led_strip: \"^2.0.0\"\n  my_local:\n    path: ../my_local\n";
    expect(
        "main/idf_component.yml",
        text,
        &[
            ("espressif/button", "^4.1.4", "esp"),
            ("lvgl/lvgl", "9.6.0", "esp"),
            ("espressif/led_strip", "^2.0.0", "esp"),
            ("my_local", "../my_local", "directory"),
        ],
    );
}

#[test]
fn reads_fpm_git_tags() {
    // The fpm manifest spec's inline and table forms; a branch or revision is not a release.
    let text = "[dependencies]\ntoml-f = { git = \"https://github.com/toml-f/toml-f\", tag = \"v0.2.1\" }\nhead = { git = \"https://github.com/toml-f/toml-f\", branch = \"main\" }\n[dependencies.stdlib]\ngit = \"https://github.com/fortran-lang/stdlib.git\"\ntag = \"v0.5.0\"\n";
    expect(
        "fpm.toml",
        text,
        &[
            ("toml-f/toml-f", "v0.2.1", "github"),
            ("fortran-lang/stdlib", "v0.5.0", "github"),
        ],
    );
}

#[test]
fn reads_boss_dependencies() {
    let text = "{\n  \"name\": \"my-project\",\n  \"version\": \"1.0.0\",\n  \"homepage\": \"https://github.com/myuser/my-project\",\n  \"dependencies\": {\n    \"github.com/HashLoad/horse\": \"^3.0.0\",\n    \"github.com/viniciussanchez/dataset-serialize\": \"v2.6.2\"\n  }\n}\n";
    expect(
        "boss.json",
        text,
        &[
            ("HashLoad/horse", "^3.0.0", "github"),
            ("viniciussanchez/dataset-serialize", "v2.6.2", "github"),
        ],
    );
}

#[test]
fn reads_mill_java_dependencies() {
    // mill-build.org's library-deps page; a Scala `::` dependency in a file with no
    // `scalaVersion` is not read.
    let text = "object foo extends JavaModule {\n  def mvnDeps = Seq(\n    mvn\"org.slf4j:slf4j-api:1.7.25\",\n    ivy\"ch.qos.logback:logback-classic:1.2.0;classifier=tests\",\n    mvn\"com.lihaoyi::pprint:0.5.3\"\n  )\n}\n";
    expect(
        "build.mill",
        text,
        &[
            ("org.slf4j:slf4j-api", "1.7.25", "maven"),
            ("ch.qos.logback:logback-classic", "1.2.0", "maven"),
        ],
    );
    // com-lihaoyi/mill's example/javalib/basic/1-simple/build.mill.yaml.
    let yaml = "extends: JavaModule\nmvnDeps:\n- net.sourceforge.argparse4j:argparse4j:0.9.0\n- org.thymeleaf:thymeleaf:3.1.1.RELEASE\n";
    expect(
        "build.mill.yaml",
        yaml,
        &[
            ("net.sourceforge.argparse4j:argparse4j", "0.9.0", "maven"),
            ("org.thymeleaf:thymeleaf", "3.1.1.RELEASE", "maven"),
        ],
    );
}

#[test]
fn reads_mill_scala_dependencies_with_the_scala_version() {
    // com-lihaoyi/mill's example/scalalib/basic/5-programmable/build.mill.
    let text = "object foo extends ScalaModule {\n  def scalaVersion = \"3.8.2\"\n  def mvnDeps = Seq(\n    mvn\"com.lihaoyi::scalatags:0.13.1\"\n  )\n}\n";
    expect(
        "build.mill",
        text,
        &[("com.lihaoyi:scalatags_3", "0.13.1", "maven")],
    );
    // scalalib/dependencies.adoc's example: `:::` appends the full Scala version, and Maven
    // Central publishes org.scalamacros:paradise_2.12.12.
    let full = "object foo extends ScalaModule {\n  def scalaVersion = \"2.12.12\"\n  def mvnDeps = Seq(\n    mvn\"com.typesafe.akka::akka-actor:2.5.25\",\n    mvn\"org.scalamacros:::paradise:2.1.1\"\n  )\n}\n";
    expect(
        "build.mill",
        full,
        &[
            ("com.typesafe.akka:akka-actor_2.12", "2.5.25", "maven"),
            ("org.scalamacros:paradise_2.12.12", "2.1.1", "maven"),
        ],
    );
    // Its 1-simple/build.mill.yaml, at Scala 2.13.
    let yaml = "extends: ScalaModule\nscalaVersion: 2.13.16\nmvnDeps:\n- com.lihaoyi::scalatags:0.13.1\n- com.lihaoyi::mainargs:0.7.8";
    expect(
        "build.mill.yaml",
        yaml,
        &[
            ("com.lihaoyi:scalatags_2.13", "0.13.1", "maven"),
            ("com.lihaoyi:mainargs_2.13", "0.7.8", "maven"),
        ],
    );
}

#[test]
fn reads_buck_archives_and_releases() {
    // sluongng/buck2-go's toolchains/BUCK and tweag/buck2.nix's examples/BUCK, whose nixpkgs
    // archive is a commit.
    let text = "http_archive(\n    name = \"bazel_rules_go\",\n    urls = [\n        \"https://github.com/bazelbuild/rules_go/releases/download/v0.40.1/rules_go-v0.40.1.zip\",\n    ],\n)\nhttp_archive(\n    name = \"nixpkgs\",\n    urls = [\n      \"https://github.com/NixOS/nixpkgs/archive/5d736263df906c5da72ab0f372427814de2f52f8.tar.gz\",\n    ],\n)\n";
    expect(
        "toolchains/BUCK",
        text,
        &[
            ("bazelbuild/rules_go", "v0.40.1", "github"),
            (
                "https://github.com/NixOS/nixpkgs",
                "5d736263df906c5da72ab0f372427814de2f52f8",
                "source",
            ),
        ],
    );
}

#[test]
fn reads_pants_requirements_and_jvm_artifacts() {
    // The Pants docs' third-party dependency examples, Python and JVM.
    let text = "python_requirement(\n    name=\"django\",\n    requirements=[\"Django==3.2.1\"],\n)\npython_requirement(\n    name=\"requests\",\n    requirements=[\"requests[security]>=1,<2.0\"],\n)\npython_requirement(\n    name=\"dataclasses\",\n    requirements=[\"dataclasses ; python_version<'3.7'\"],\n)\njvm_artifact(\n    group=\"com.google.guava\",\n    artifact=\"guava\",\n    version=\"31.0.1-jre\",\n    packages=[\"com.google.common.**\"],\n)\n";
    expect(
        "src/python/BUILD",
        text,
        &[
            ("Django", "==3.2.1", "pypi"),
            ("requests", ">=1,<2.0", "pypi"),
            ("com.google.guava:guava", "31.0.1-jre", "maven"),
        ],
    );
}

#[test]
fn reads_west_projects() {
    // The west manifest docs' projects: a tagged `url` project in each field order, a commit, a
    // pull ref, a remote's tagged project with a `repo-path`, and one through `defaults.remote`.
    let text = "manifest:\n  defaults:\n    remote: remote1\n  remotes:\n    - name: remote1\n      url-base: https://github.com/zephyrproject-rtos\n    - name: remote2\n      url-base: https://git.example.com/base2/\n  projects:\n    - name: zephyr\n      url: https://github.com/zephyrproject-rtos/zephyr\n      revision: v3.7.0\n      import: true\n    - name: hal\n      revision: v1.2.0\n      url: https://github.com/user/hal.git\n    - name: proj2\n      remote: remote1\n      repo-path: hal_nordic\n      revision: v1.3\n    - name: proj3\n      url: https://github.com/user/project-three\n      revision: 8d24867bc9c9d81c81eeac59391cda59333affd4\n    - name: proj4\n      url: https://github.com/user/project-four\n      revision: pull/69/head\n    - name: proj5\n      revision: main\n    - name: proj6\n      remote: remote2\n      revision: v2.0.0\n    - name: proj7\n      remote: remote1\n";
    expect(
        "west.yml",
        text,
        &[
            ("zephyrproject-rtos/zephyr", "v3.7.0", "github"),
            ("user/hal", "v1.2.0", "github"),
            ("zephyrproject-rtos/hal_nordic", "v1.3", "github"),
            (
                "https://github.com/user/project-three",
                "8d24867bc9c9d81c81eeac59391cda59333affd4",
                "source",
            ),
            (
                "https://github.com/user/project-four",
                "pull/69/head",
                "source",
            ),
            (
                "https://github.com/zephyrproject-rtos/proj5",
                "main",
                "source",
            ),
            ("https://git.example.com/base2/proj6", "v2.0.0", "source"),
        ],
    );
}

#[test]
fn reads_alire_dependencies() {
    // The Alire catalog spec's static and dynamic dependencies.
    let text = "name = \"my_crate\"\nversion = \"0.1.0\"\n\n[[depends-on]]\nlibfoo = \"^1.2\"\naunit = \"*\"\n\n[[depends-on]]\n[depends-on.'case(os)'.linux]\nlibbar = \"^2.0\"\n";
    expect(
        "alire.toml",
        text,
        &[
            ("libfoo", "^1.2", "alire"),
            ("aunit", "*", "alire"),
            ("libbar", "^2.0", "alire"),
        ],
    );
}

#[test]
fn reads_arduino_library_depends() {
    // arduino-libraries/ArduinoIoTCloud's library.properties, with the spec's version constraint.
    let text = "name=ArduinoIoTCloud\nversion=2.10.0\narchitectures=mbed,samd,esp8266\nincludes=ArduinoIoTCloud.h\ndepends=Arduino_ConnectionHandler,Adafruit SleepyDog Library, ArduinoHttpClient (>=0.6.0),ArduinoBearSSL\n";
    expect(
        "library.properties",
        text,
        &[
            ("Arduino_ConnectionHandler", "", "arduino"),
            ("Adafruit SleepyDog Library", "", "arduino"),
            ("ArduinoHttpClient", ">=0.6.0", "arduino"),
            ("ArduinoBearSSL", "", "arduino"),
        ],
    );
}

#[test]
fn reads_nimble_package_names() {
    // status-im/nim-chronos's chronos.nimble and karaxnim/karax's karax.nimble; `nim` is the
    // compiler, and a `skipDirs` list is not read.
    let text = "version       = \"4.4.0\"\ndescription   = \"Networking framework with async/await support\"\nskipDirs      = @[\"tests\"]\n\nrequires \"nim >= 1.6.16\",\n         \"bearssl >= 0.2.13\",\n         \"stew >= 0.5.0\"\nrequires \"ws\"\nrequires \"dotenv >= 2.0.0\"\nrequires \"foo 1.0.0\"\nrequires \"https://github.com/user/pkg#head\"\nbin = @[\"karax/tools/karun\"]\n";
    expect(
        "chronos.nimble",
        text,
        &[
            ("nim-lang/nim", ">= 1.6.16", "github"),
            ("bearssl", ">= 0.2.13", "nimble"),
            ("stew", ">= 0.5.0", "nimble"),
            ("ws", "", "nimble"),
            ("dotenv", ">= 2.0.0", "nimble"),
            ("foo", "1.0.0", "nimble"),
            ("https://github.com/user/pkg", "head", "source"),
        ],
    );
}

#[test]
fn reads_github_commit_archives_as_sources() {
    // A build.zig.zon dependency pinned to a commit archive beside one at a tag archive.
    let text = ".{\n    .dependencies = .{\n        .known = .{\n            .url = \"https://github.com/o/known/archive/refs/tags/v1.2.0.tar.gz\",\n        },\n        .pinned = .{\n            .url = \"https://github.com/o/pinned/archive/0123456789abcdef0123456789abcdef01234567.tar.gz\",\n        },\n    },\n}\n";
    expect(
        "build.zig.zon",
        text,
        &[
            ("o/known", "v1.2.0", "github"),
            (
                "https://github.com/o/pinned",
                "0123456789abcdef0123456789abcdef01234567",
                "source",
            ),
        ],
    );
}
