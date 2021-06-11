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

FROM ubuntu
ARG PROFILE="release"
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get update && apt-get install curl ca-certificates npm nodejs -y
RUN npm install -g serve
COPY --from=builder /kylin-node /
COPY --from=builder /polkadot /
COPY --from=builder /subkey /
COPY --from=builder /builds/kylin-node/scripts/insert_alice_key.sh /
COPY --from=builder /builds/kylin-node/scripts/rococo-local.json /
COPY --from=builder /builds/sample-data-fetcher/data/es_index.json /es_index.json
COPY --from=builder /builds/sample-data-fetcher/data/kibana.ndjson /kibana.ndjson
COPY --from=builder /builds/sample-data-fetcher/data/init_es_index.sh /init_es_index.sh
COPY --from=builder /builds/sample-data-fetcher/data/init_kibana_dashboard.sh /init_kibana_dashboard.sh

COPY --from=builder /builds/apps/packages/apps /apps
#
EXPOSE 30330-30345 9933-9960 8080 3001
