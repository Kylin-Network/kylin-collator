# build kylin-node
FROM kylinnetwork/polkadot:latest as builder

LABEL maintainer="kylin-dev"
ARG RUST_VERSION=1.51.0
ARG PROFILE=release
ARG KYLIN_GIT_REPO="https://github.com/Kylin-Network/kylin-node.git"
ARG DOT_GIT_REPO="https://github.com/paritytech/polkadot.git"
ARG GIT_BRANCH="polkadot-master"
WORKDIR /builds/

#Build Kylin-node
RUN git clone --recursive ${KYLIN_GIT_REPO}

WORKDIR /builds/kylin-node
RUN git checkout ${GIT_BRANCH}
RUN git submodule update --recursive --remote
RUN cargo build --${PROFILE}
RUN cp target/${PROFILE}/kylin-node /kylin-node

RUN /polkadot build-spec --chain rococo-local --disable-default-bootnode --raw > rococo-local.json

FROM ubuntu
ENV DEBIAN_FRONTEND noninteractive
RUN apt update && apt install git gnupg2 curl ca-certificates vim npm nodejs -y
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - && echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list

LABEL maintainer="kylin-dev"
ARG FRONT_REPO="https://github.com/Kylin-Network/apps.git"
ARG FRONT_BRANCH="master"

WORKDIR /builds/
RUN git clone --recursive ${FRONT_REPO}
WORKDIR /builds/apps


RUN apt update && apt install yarn -y
RUN apt purge nodejs npm -y

RUN apt-get install -y \
    software-properties-common \
    npm

RUN npm install npm@latest -g && \
    npm install n -g && \
    n latest
RUN YARN_CHECKSUM_BEHAVIOR=update yarn && yarn build:www

RUN npm install -g serve

ARG POLKADOT_LAUNCH_REPO="https://github.com/Kylin-Network/polkadot-launch.git"
ARG DATA_FETECHER_BRANCH="master"

WORKDIR /builds/
RUN git clone --recursive ${POLKADOT_LAUNCH_REPO}
WORKDIR /builds/polkadot-launch
RUN yarn global add polkadot-launch

COPY --from=builder /kylin-node /
COPY --from=builder /polkadot /
COPY --from=builder /subkey /
COPY --from=builder /builds/kylin-node/scripts/insert_alice_key.sh /
COPY --from=builder /builds/kylin-node/rococo-local.json /
COPY --from=builder /builds/sample-data-fetcher/data/es_index.json /es_index.json
COPY --from=builder /builds/sample-data-fetcher/data/kibana.ndjson /kibana.ndjson
COPY --from=builder /builds/sample-data-fetcher/data/init_es_index.sh /init_es_index.sh
COPY --from=builder /builds/sample-data-fetcher/data/init_kibana_dashboard.sh /init_kibana_dashboard.sh
COPY --from=builder /builds/polkadot-launch/ polkadot-launch/

COPY --from=builder /builds/apps/packages/apps /apps
WORKDIR /
#
EXPOSE 30330-30345 9933-9960 8080 3001
