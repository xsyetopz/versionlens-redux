# see ./publish.gitlab-ci.yml for cmdline example of how to run on local docker
FROM node:22-alpine3.19
ARG TARGET_PATH=/versionlens

COPY / $TARGET_PATH

WORKDIR $TARGET_PATH

# install deps
RUN npm install -g npm @vscode/vsce

RUN npm install

# run tests
RUN npm run test

# bundle
RUN npm run bundle

# package the extension
RUN vsce package

# move package to artifacts
RUN mkdir ./artifacts
RUN mv *.vsix ./artifacts

# publish the package
CMD vsce publish