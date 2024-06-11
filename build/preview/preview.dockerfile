# see ./preview.gitlab-ci.yml for cmdline example of how to run on local docker
FROM node:22-alpine3.19
ARG TARGET_PATH=/versionlens

COPY / $TARGET_PATH

WORKDIR $TARGET_PATH

# install deps
RUN npm install -g npm @vscode/vsce js-build-tasks

RUN npm install

# run tests
RUN task build:test

# bundle
RUN task bundle

# set preview=true in package.json
RUN node -e "const {readFileSync, writeFileSync} = require('fs'); const pkg = JSON.parse(readFileSync('./package.json', 'utf-8')); pkg.preview=true; pkg.version = pkg.version.replace('-preview', ''); writeFileSync('./package.json', JSON.stringify(pkg,null,'  '))"

# package the extension
RUN vsce package --pre-release

# move package to artifacts
RUN mkdir ./artifacts
RUN mv *.vsix ./artifacts

# publish the package
CMD vsce publish --pre-release