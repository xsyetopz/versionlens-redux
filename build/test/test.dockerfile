# see ./docker.test.tasks.yml on how this container is created
FROM node:20.18-alpine
ARG TARGET_PATH=/versionlens

# update os packages
RUN apk update && apk upgrade

# copy in project files (minus the .dockerignore entries)
COPY / $TARGET_PATH

# set the $CWD to the project root
WORKDIR $TARGET_PATH

# update npm to latest
RUN npm install -g npm js-build-tasks

# install dependencies
RUN npm ci

# run tests
CMD task coverage