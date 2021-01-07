# build kylin-node
FROM paritytech/ci-linux:production as builder

LABEL maintainer="kylin-dev"
ARG RUST_VERSION=nightly-2020-10-06
ARG PROFILE=debug
ARG GIT_REPO="https://github.com/Kylin-Network/kylin-node.git"
ARG GIT_BRANCH="master"

RUN git clone --recursive ${GIT_REPO}
WORKDIR /builds/kylin-node
RUN sed -i s/localhost/kylin-data-proxy/g /builds/kylin-node/pallets/data-fetcher/src/lib.rs
RUN sed -i s/localhost/kylin-node/g /builds/kylin-node/scripts/insert_alice_key.sh
RUN rustup default stable
RUN rustup uninstall nightly
RUN rustup toolchain install ${RUST_VERSION}
RUN rustup target add wasm32-unknown-unknown --toolchain ${RUST_VERSION}

RUN cargo build
RUN cp target/${PROFILE}/kylin-node /kylin-node

# build sample data fetcher
FROM paritytech/ci-linux:production as data_fetcher_builder
ARG DATA_FETECHER_REPO="https://github.com/Kylin-Network/sample-data-fetcher.git"
ARG DATA_FETECHER_BRANCH="master"
ARG PROFILE="debug"

RUN git clone --recursive ${DATA_FETECHER_REPO}
WORKDIR /builds/sample-data-fetcher
RUN cargo build
RUN cp target/${PROFILE}/data_fetcher /data_fetcher

FROM ubuntu
RUN apt-get update && apt-get install curl ca-certificates -y
COPY --from=builder /kylin-node /
COPY --from=builder /builds/kylin-node/scripts/insert_alice_key.sh /

COPY --from=data_fetcher_builder /builds/sample-data-fetcher/target/debug/data_fetcher /data_fetcher

EXPOSE 30333 9933 9944 8080
