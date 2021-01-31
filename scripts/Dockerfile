# build kylin-node
FROM paritytech/ci-linux:production as builder

LABEL maintainer="kylin-dev"
ARG RUST_VERSION=nightly-2021-01-29
ARG PROFILE=release
ARG GIT_REPO="https://github.com/Kylin-Network/kylin-node.git"
ARG GIT_BRANCH="master"

RUN git clone --recursive ${GIT_REPO}
WORKDIR /builds/kylin-node
RUN sed -i s/localhost/kylin-data-proxy/g /builds/kylin-node/pallets/kylin-ocw/src/lib.rs
RUN sed -i s/localhost/kylin-node/g /builds/kylin-node/scripts/insert_alice_key.sh


RUN rustup default stable
RUN rustup uninstall nightly
RUN rustup toolchain install ${RUST_VERSION}
RUN rustup target add wasm32-unknown-unknown --toolchain ${RUST_VERSION}



RUN sed -i 's/2.0.0/2.0.1/g'  /builds/kylin-node/node/Cargo.toml
RUN sed -i 's/0.8.0/0.8.1/g'  /builds/kylin-node/node/Cargo.toml

RUN sed -i 's/2.0.0/2.0.1/g'  /builds/kylin-node/runtime/Cargo.toml
RUN sed -i 's/0.8.0/0.8.1/g'  /builds/kylin-node/runtime/Cargo.toml

RUN sed -i 's/2.0.0/2.0.1/g'  /builds/kylin-node/pallets/kylin-ocw/Cargo.toml
RUN sed -i 's/0.8.0/0.8.1/g'  /builds/kylin-node/pallets/kylin-ocw/Cargo.toml

RUN sed -i 's/2.0.0/2.0.1/g'  /builds/kylin-node/pallets/kylin-oracle/Cargo.toml
RUN sed -i 's/0.8.0/0.8.1/g'  /builds/kylin-node/pallets/kylin-oracle/Cargo.toml

RUN cargo +${RUST_VERSION} build --release || true && curl https://raw.githubusercontent.com/paritytech/substrate/master/primitives/arithmetic/src/fixed_point.rs > /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/sp-arithmetic-2.0.0/src/fixed_point.rs
RUN sed -i 's/1.into()/1u32.into()/g' /builds/kylin-node/pallets/kylin-oracle/src/lib.rs
RUN sed -i 's/let one_thousand = NumberFor::<B>::from(1_000)/let one_thousand = NumberFor::<B>::from(1_000u32)/g' /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/sc-informant-0.8.0/src/display.rs
RUN cargo +${RUST_VERSION} build --release
#RUN curl https://raw.githubusercontent.com/paritytech/substrate/master/primitives/arithmetic/src/fixed_point.rs > /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/sp-arithmetic-2.0.0/src/fixed_point.rs
#RUN sed -i 's/let public = generic_public.into()/let public: T::Public = generic_public.into()/g'  /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/frame-system-2.0.0/src/offchain.rs

#
##RUN curl https://raw.githubusercontent.com/paritytech/substrate/master/frame/randomness-collective-flip/src/lib.rs > /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/pallet-randomness-collective-flip-2.0.0/src/lib.rs

#RUN cargo +${RUST_VERSION} build --release |

#RUST_VERSION=nightly-2021-01-29;GIT_REPO="https://github.com/Kylin-Network/kylin-node.git";git clone --recursive ${GIT_REPO};cd  /builds/kylin-node;rustup uninstall nightly;rustup install ${RUST_VERSION};rustup target add wasm32-unknown-unknown --toolchain ${RUST_VERSION};
#RUST_VERSION=nightly-2021-01-29;rustup uninstall nightly;rustup install ${RUST_VERSION};rustup target add wasm32-unknown-unknown --toolchain ${RUST_VERSION};cargo +${RUST_VERSION} build --release


#RUN sed -i 's/1.into()/One::one()/g' /builds/kylin-node/pallets/kylin-oracle/src/lib.rs
#RUN sed -i 's/1.into()/sp_runtime::traits::One::one()/g' /builds/kylin-node/pallets/kylin-oracle/src/lib.rs
#
#RUN sed -i 's/1.into()/One::one()/g' /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/pallet-randomness-collective-flip-2.0.0/src/lib.rs
#RUN sed -i 's/1.into()/sp_runtime::traits::One::one()/g' /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/pallet-randomness-collective-flip-2.0.0/src/lib.rs
#RUN sed -i 's/2.into()/sp_runtime::traits::One::one()/g' /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/pallet-grandpa-2.0.0/src/lib.rs
#RUN sed -i 's/2.into()/sp_runtime::traits::One::one()/g' /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/pallet-aura-2.0.0/src/lib.rs
#RUN sed -i 's/0.into()/sp_runtime::traits::Zero::zero()/g' /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/pallet-timestamp-2.0.0/src/lib.rs
#RUN sed -i 's/0.into()/sp_runtime::traits::Zero::zero()/g' /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/pallet-aura-2.0.0/src/lib.rs
#RUN sed -i 's/20.into()/sp_runtime::traits::One::one()/g' /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/sc-transaction-pool-2.0.0/src/lib.rs
#RUN sed -i 's/.into()/u32.into()/g' /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/sc-service-0.8.0/src/chain_ops/export_blocks.rs
#
#
#RUN sed -i 's/1.into()/One::one()/g' /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/frame-executive-2.0.0/src/lib.rs
#RUN sed -i 's/1.into()/sp_runtime::traits::One::one()/g' /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/frame-executive-2.0.0/src/lib.rs
#
#









#RUN wget https://raw.githubusercontent.com/paritytech/substrate/master/frame/system/src/offchain.rs /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/frame-system-2.0.0/src/offchain.rs;cp offchain.rs /usr/local/cargo/registry/src/github.com-1ecc6299db9ec823/frame-system-2.0.0/src/offchain.rs


#RUN cargo +${RUST_VERSION} build
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

# build kylin-market-frontend
FROM paritytech/ci-linux:production as frontend_builder

LABEL maintainer="kylin-dev"
ARG FRONT_REPO="https://github.com/Kylin-Network/kylin-market-frontend.git"
ARG FRONT_BRANCH="master"

RUN git clone --recursive ${FRONT_REPO}
WORKDIR /builds/kylin-market-frontend

RUN apt update && apt install gnupg2 curl ca-certificates -y
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - && echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list
RUN apt update && apt install yarn -y
RUN YARN_CHECKSUM_BEHAVIOR=update yarn && yarn build:www

FROM ubuntu
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get update && apt-get install curl ca-certificates npm nodejs -y
RUN npm install -g serve
COPY --from=builder /kylin-node /
COPY --from=builder /builds/kylin-node/scripts/insert_alice_key.sh /

COPY --from=data_fetcher_builder /builds/sample-data-fetcher/target/debug/data_fetcher /data_fetcher
COPY --from=data_fetcher_builder /builds/sample-data-fetcher/data/es_index.json /es_index.json
COPY --from=data_fetcher_builder /builds/sample-data-fetcher/data/kibana.ndjson /kibana.ndjson
COPY --from=data_fetcher_builder /builds/sample-data-fetcher/data/init_es_index.sh /init_es_index.sh
COPY --from=data_fetcher_builder /builds/sample-data-fetcher/data/init_kibana_dashboard.sh /init_kibana_dashboard.sh

COPY --from=frontend_builder /builds/kylin-market-frontend/packages/apps /apps

EXPOSE 30333 9933 9944 8080 3001
