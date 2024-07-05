# see ./docker.preview.tasks.yml on how this container is created
FROM node:current-alpine
ARG TARGET_PATH=/versionlens
ENV PREVIEW_OUT_PATH=.preview

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
RUN task preview:prepack

# create the artifacts folder
RUN mkdir $PREVIEW_OUT_PATH

# package and publish
CMD vsce package --pre-release --out $PREVIEW_OUT_PATH \
    && vsce publish --pre-release --packagePath $(find $PREVIEW_OUT_PATH/*.vsix)