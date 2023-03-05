FROM debian:bullseye-slim

# All this commands are in the same layer to reduce the image size
# Install :
#  - Vault       https://developer.hashicorp.com/vault/docs/commands
#  - kubectl     https://kubernetes.io/docs/tasks/tools/install-kubectl-linux/
#  - jq          https://manpages.org/jq
RUN apt-get update &&\
    apt-get install -y wget unzip jq &&\
    wget -q "https://releases.hashicorp.com/vault/1.10.3/vault_1.10.3_linux_amd64.zip" -O /tmp/vault.zip &&\
    unzip -d /usr/bin /tmp/vault.zip &&\
    chmod 555 /usr/bin/vault &&\
    wget -q "https://dl.k8s.io/release/v1.26.2/bin/linux/amd64/kubectl" -O /usr/bin/kubectl &&\
    chmod 555 /usr/bin/kubectl &&\
    apt-get purge -y wget unzip &&\
    apt-get autoremove -y &&\
    apt-get clean &&\
    rm -rf /var/lib/apt/lists/ &&\
    rm -rf /tmp/* &&\
    rm -rf /var/tmp/* &&\
    useradd -m 1000 -s /bin/bash

USER 1000
WORKDIR /home/1000

ADD --chown=1000:1000 entrypoint.sh LICENCE /home/1000/

CMD ["/home/1000/entrypoint.sh"]
