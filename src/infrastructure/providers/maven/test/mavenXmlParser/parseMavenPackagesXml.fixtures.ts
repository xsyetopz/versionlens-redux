export default {

  parseMavenPackagesXml: {

    test: `
      <project>
        <modelVersion>4.0.0</modelVersion>
        <groupId>vscode-contrib</groupId>
        <artifactId>vscode-versionlens</artifactId>
        <packaging>war</packaging>
        <version>1.3.6-SNAPSHOT</version>
        <name>smoke-test</name>
        <properties>
          <tomcat.groupId>org.apache.tomcat</tomcat.groupId>
          <tomcat.artifactId>tomcat</tomcat.artifactId>
          <tomcat.version>9.0.12</tomcat.version>
        </properties>
        <parent>
          <groupId>org.springframework.boot</groupId>
          <artifactId>spring-boot-starter-parent</artifactId>
          <version>1.5.16.RELEASE</version>
        </parent>
        <dependencies>
          <dependency>
            <groupId>org.springframework</groupId>
            <artifactId>spring-core</artifactId>
            <version>5.0.7.RELEASE</version>
          </dependency>
          <dependency>
            <groupId>org.apache.tomcat</groupId>
            <artifactId>\${tomcat.artifactId}</artifactId>
            <version>\${tomcat.version}</version>
            <type>pom</type>
          </dependency>
        </dependencies>
        <repositories>
          <repository>
            <url>https://packages.atlassian.com/maven-3rdparty/</url>
          </repository>
        </repositories>
      </project>
    `,

    expected: [
      {
        "types": {
          "name": {
            "type": "name",
            "name": "org.springframework.boot:spring-boot-starter-parent",
            "nameRange": {
              "start": 481,
              "end": 481
            }
          },
          "version": {
            "type": "version",
            "version": "1.5.16.RELEASE",
            "versionAppend": "",
            "versionPrepend": "",
            "versionRange": {
              "start": 625,
              "end": 639
            },
          }
        },
        "typeCount": 2
      },
      {
        "types": {
          "name": {
            "type": "name",
            "name": "org.springframework:spring-core",
            "nameRange": {
              "start": 701,
              "end": 701
            }
          },
          "version": {
            "type": "version",
            "version": "5.0.7.RELEASE",
            "versionAppend": "",
            "versionPrepend": "",
            "versionRange": {
              "start": 835,
              "end": 848
            },
          }
        },
        "typeCount": 2
      },
      {
        "types": {
          "name": {
            "type": "name",
            "name": "org.apache.tomcat:tomcat",
            "nameRange": {
              "start": 893,
              "end": 893
            }
          },
          "version": {
            "type": "version",
            "version": "9.0.12",
            "versionAppend": "",
            "versionPrepend": "",
            "versionRange": {
              "start": 427,
              "end": 433
            },
          }
        },
        "typeCount": 2
      }
    ]
  },

  extractReposUrlsFromXml: {
    test: `
      <?xml version="1.0" encoding="UTF-8"?>
      <settings xmlns="http://maven.apache.org/SETTINGS/1.0.0" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://maven.apache.org/SETTINGS/1.0.0 https://maven.apache.org/xsd/settings-1.0.0.xsd">
        <localRepository>C:\\Users\\UserName\\.m2\\repository</localRepository>
        <profiles>
          <profile>
            <id>adobe-public</id>
            <repositories>
              <repository>
                <id>adobe-public-releases</id>
                <name>Adobe Public Repository</name>
                <url>https://repo.adobe.com/nexus/content/groups/public</url>
                <releases>
                  <enabled>true</enabled>
                  <updatePolicy>never</updatePolicy>
                </releases>
                <snapshots>
                  <enabled>false</enabled>
                </snapshots>
              </repository>
            </repositories>
          </profile>
        </profiles>
      </settings>`,
    expected: [
      "C:\\Users\\UserName\\.m2\\repository",
      "https://repo.adobe.com/nexus/content/groups/public"
    ]
  },

  getVersionsFromPackageXml: {
    test: `
      <?xml version="1.0" encoding="UTF-8"?>
      <metadata>
        <versioning>
          <versions>
            <version>1.2.3</version>
            <version>1.2.4</version>
            <version>1.2.5</version>
          </versions>
        </versioning>
      </metadata>
    `,
    expected: [
      '1.2.3',
      '1.2.4',
      '1.2.5'
    ]
  }
}