# see ./tasks.yml for example of how to run on local docker
FROM node:22-alpine3.19
ARG TARGET_PATH=/versionlens

COPY / $TARGET_PATH

WORKDIR $TARGET_PATH

RUN npm install -g npm js-build-tasks

RUN npm install

CMD task test