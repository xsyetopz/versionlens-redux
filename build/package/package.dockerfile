# see ./docker.package.tasks.yml on how this container is created
FROM node:20.18-alpine
ARG TARGET_PATH=/versionlens
ENV PACKAGE_OUT_PATH=.package

# update os packages
RUN apk update && apk upgrade

# copy in project files (minus the .dockerignore entries)
COPY / $TARGET_PATH

# set the $CWD to the project root
WORKDIR $TARGET_PATH

# update npm to latest
RUN npm install -g npm @vscode/vsce js-build-tasks

# install dependencies
RUN npm ci

# run tests
RUN task build:test

# bundle
RUN task bundle

# create artifacts folder
RUN mkdir $PACKAGE_OUT_PATH

# package vsix and move it to artifacts folder
CMD vsce package --out $PACKAGE_OUT_PATH