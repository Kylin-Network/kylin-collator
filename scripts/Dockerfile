FROM paritytech/ci-linux:production as builder

LABEL maintainer="kylin-dev"
ARG RUST_VERSION=nightly-2020-10-06
ARG PROFILE=debug
ARG GIT_REPO="https://github.com/Kylin-Network/kylin-node.git"
ARG GIT_BRANCH="master"

RUN git clone --recursive ${GIT_REPO}
WORKDIR /builds/kylin-node
RUN rustup default stable
RUN rustup uninstall nightly
RUN rustup toolchain install ${RUST_VERSION}
RUN rustup target add wasm32-unknown-unknown --toolchain ${RUST_VERSION}

RUN cargo build
RUN cp target/${PROFILE}/kylin-node /kylin-node

FROM ubuntu
COPY --from=builder /kylin-node /
EXPOSE 30333 9933 9944