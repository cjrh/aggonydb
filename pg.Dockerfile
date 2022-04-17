FROM postgres:14.2-bullseye

# Change the default Docker shell + enable bash strictmode
# http://redsymbol.net/articles/unofficial-bash-strict-mode
SHELL ["/usr/bin/env", "bash", "-o", "xtrace", "-o", "errexit", "-o", "nounset", "-o", "pipefail", "-c"]

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update; \
    apt install -yq \
        ca-certificates \
        postgresql-14-hll \
    ; \
    rm -rf /var/lib/apt/lists/*; \
    echo 'install dependencies: OK'
