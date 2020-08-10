FROM ubuntu:18.04 as builder
ARG symbol=SYS

RUN apt update

RUN apt install cmake git curl make bzip2 automake libbz2-dev libssl-dev doxygen graphviz libgmp3-dev autotools-dev \
	libicu-dev python2.7 python2.7-dev python3 python3-dev autoconf libtool zlib1g-dev ruby libusb-1.0-0-dev libcurl4-gnutls-dev \
	pkg-config patch sudo llvm-7 -y

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
	export PATH="$PATH:$HOME/.cargo/bin"

RUN git clone https://github.com/bifrost-codes/bifrost-eos-relay.git --recursive
RUN cd bifrost-eos-relay && git checkout master && git submodule update --init --recursive
RUN git config --global user.email "you@example.com" && git config --global user.name "Your Name"
RUN apt install wget -y && wget https://dl.bintray.com/boostorg/release/1.70.0/source/boost_1_70_0.tar.bz2 \
	&& tar -xvf boost_1_70_0.tar.bz2 && cd boost_1_70_0 && \
	./bootstrap.sh --prefix=/usr/local && ./b2 && ./b2 install

RUN cd bifrost-eos-relay && git tag v2.0.4 -m 'fix no tag' && c && \
	cmake -DCMAKE_BUILD_TYPE=Release -DCORE_SYMBOL_NAME=$symbol .. \
	&& export PATH="$PATH:$HOME/.cargo/bin" && make -j4

FROM ubuntu:18.04

RUN apt-get update && \
	apt-get install -y openssl ca-certificates

COPY --from=builder /bifrost-eos-relay/build/bin/nodeos /usr/local/bin
COPY --from=builder /bifrost-eos-relay/build/plugins/bridge_plugin/librpc_client.so /usr/lib

EXPOSE 8888 8889 9876 9877
ENTRYPOINT ["/usr/local/bin/nodeos"]

CMD ["/usr/local/bin/nodeos"]
ENV DEBIAN_FRONTEND teletype
