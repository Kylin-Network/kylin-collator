# build kylin-collator
FROM kylinnetwork/polkadot:latest as builder

LABEL maintainer="kylin-dev"
ARG RUST_VERSION=1.51.0
ARG PROFILE=release
ARG KYLIN_GIT_REPO="https://github.com/Kylin-Network/kylin-collator.git"
ARG GIT_BRANCH="main"
WORKDIR /builds/

#Build kylin-collator
RUN git clone --recursive ${KYLIN_GIT_REPO}

WORKDIR /builds/kylin-collator
RUN git checkout ${GIT_BRANCH}
RUN cargo build --${PROFILE}
RUN cp target/${PROFILE}/kylin-collator /kylin-collator

FROM ubuntu
ENV DEBIAN_FRONTEND noninteractive
RUN apt update && apt install git gnupg2 curl ca-certificates vim npm nodejs wget awscli -y
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
    n stable

RUN YARN_CHECKSUM_BEHAVIOR=update yarn && yarn
RUN npm install -g serve

ARG POLKADOT_LAUNCH_REPO="https://github.com/Kylin-Network/polkadot-launch.git"

WORKDIR /builds/
RUN git clone --recursive ${POLKADOT_LAUNCH_REPO}
WORKDIR /builds/polkadot-launch
RUN yarn global add polkadot-launch


WORKDIR /
COPY --from=builder /kylin-collator /
COPY --from=builder /polkadot /
COPY --from=builder /subkey /


EXPOSE 30330-30345 9933-9960 8080 3001