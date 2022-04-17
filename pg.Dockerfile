FROM postgres:14.2-bullseye

# Change the default Docker shell + enable bash strictmode
# http://redsymbol.net/articles/unofficial-bash-strict-mode
SHELL ["/usr/bin/env", "bash", "-o", "xtrace", "-o", "errexit", "-o", "nounset", "-o", "pipefail", "-c"]

ENV DEBIAN_FRONTEND=noninteractive

# Configure apt
#RUN echo 'APT::Install-Recommends "false";' | tee -a /etc/apt/apt.conf.d/99-install-suggests-recommends; \
#    echo 'APT::Install-Suggests "false";' | tee -a /etc/apt/apt.conf.d/99-install-suggests-recommends; \
#    echo 'Configuring apt: OK';

RUN apt update; \
    apt install -yq \
#        build-essential \
        ca-certificates \
#        git \
#        postgresql-server-dev-13 \
#        postgresql-13-ip4r \
        postgresql-14-hll \
    ; \
    rm -rf /var/lib/apt/lists/*; \
    echo 'install dependencies: OK'

#RUN git clone https://github.com/citusdata/postgresql-hll --depth=1; \
#    cd postgresql-hll; \
#    make CC=gcc CXX=gcc; \
#    make install; \
#    make clean; \
#    cd / ; \
#    rm -rf postgresql-hll; \
#    echo "postgresql-hll installation: OK"