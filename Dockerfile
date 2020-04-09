FROM ubuntu:18.04
LABEL description="Build EOS image."

RUN apt-get update && \
    apt-get install -y openssl ca-certificates

COPY ./build/bin/nodeos /usr/local/bin
COPY ./build/plugins/bridge_plugin/librpc_client.so /usr/lib

EXPOSE 8888 8889 9876 9877
ENTRYPOINT ["/usr/local/bin/nodeos"]

CMD ["/usr/local/bin/nodeos"]
ENV DEBIAN_FRONTEND teletype
