# see ../docker.tasks.yml on how this container is created
FROM node:20.18-alpine
ARG TARGET_PATH=/versionlens
ENV PRERELEASE_OUT_PATH=.prerelease

# update os packages
RUN apk update && apk upgrade

# copy in project files (minus the .dockerignore entries)
COPY / $TARGET_PATH

# set the $CWD to the project root
WORKDIR $TARGET_PATH

# update npm to latest
RUN npm install -g npm @vscode/vsce ovsx js-build-tasks

# install dependencies
RUN npm ci

# run tests
RUN task build:test

# bundle
RUN task bundle

# create the artifacts folder
RUN mkdir $PRERELEASE_OUT_PATH

# package and publish
CMD vsce package --pre-release --out $PRERELEASE_OUT_PATH \
    && ovsx publish $(find $PRERELEASE_OUT_PATH/*.vsix)