# see ./tasks.yml for example of how to run on local docker
FROM node:22-alpine3.19
ARG TARGET_PATH=/versionlens

COPY / $TARGET_PATH

WORKDIR $TARGET_PATH

# install deps
RUN npm install -g npm @vscode/vsce

RUN npm install

# run tests
RUN npm run test:unit

# bundle
RUN npm run bundle

# create artifacts folder
RUN mkdir ./artifacts

# package vsix and move it to artifacts folder
CMD vsce package && mv *.vsix ./artifacts