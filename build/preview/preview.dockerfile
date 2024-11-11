# see ./docker.preview.tasks.yml on how this container is created
FROM node:20.18-alpine
ARG TARGET_PATH=/versionlens
ENV PREVIEW_OUT_PATH=.preview

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

# set preview=true in package.json
RUN task preview:prepack

# create the artifacts folder
RUN mkdir $PREVIEW_OUT_PATH

# package and publish
CMD vsce package --pre-release --out $PREVIEW_OUT_PATH \
    && vsce publish --pre-release --packagePath $(find $PREVIEW_OUT_PATH/*.vsix)