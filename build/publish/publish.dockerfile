# see ./publish.gitlab-ci.yml for cmdline example of how to run on local docker
FROM node:22-alpine3.19
ARG TARGET_PATH=/versionlens

COPY / $TARGET_PATH

WORKDIR $TARGET_PATH

# install deps
RUN npm install -g npm @vscode/vsce rimraf
RUN npm install

# run tests
RUN npm run test:unit

# bundle
RUN npm run bundle

# remove all node modules
RUN rimraf ./node_modules

# install prod dependencies only
RUN npm install --omit dev

# package the extension
RUN vsce package

# move package to artifacts
RUN mkdir ./artifacts
RUN mv *.vsix ./artifacts

# publish
CMD vsce publish