//! Build systems: CPM.cmake, `FetchContent`, Hunter, `PlatformIO`, ESP-IDF components, Mill, fpm, and Boss.
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
            ("bblanchon/ArduinoJson", "~5.6", "platformio"),
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
    // mill-build.org's library-deps page; a Scala `::` dependency is not read.
    let text = "object foo extends JavaModule {\n  def mvnDeps = Seq(\n    mvn\"org.slf4j:slf4j-api:1.7.25\",\n    ivy\"ch.qos.logback:logback-classic:1.2.0;classifier=tests\",\n    mvn\"com.lihaoyi::pprint:0.5.3\"\n  )\n}\n";
    expect(
        "build.mill",
        text,
        &[
            ("org.slf4j:slf4j-api", "1.7.25", "maven"),
            ("ch.qos.logback:logback-classic", "1.2.0", "maven"),
        ],
    );
}
