# 茶馆 Relay · Docker image
#
#   docker build -f deploy/docker/Dockerfile -t teahouse-relay .
#   docker run -d --name teahouse -p 2222:22 -v teahouse:/var/lib/teahouse --restart unless-stopped teahouse-relay
#   docker exec -u teahouse teahouse teahouse-relay open wutong --name "梧桐里三号院" \
#       --host-id chenli --host-name "陈立" --host-key "ssh-ed25519 AAAA…"
#
# Everything lives in the volume (/var/lib/teahouse): teahouses, attachments,
# invitations, audit log, the relay's signing key and sshd's host key — so the
# container can be replaced (upgraded) without changing anyone's identity.

FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p teahouse-relay

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends openssh-server openssh-client git ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --home-dir /var/lib/teahouse --create-home --shell /bin/sh teahouse \
 && usermod -p '*' teahouse \
 && rm -f /etc/ssh/ssh_host_*
COPY --from=build /src/target/release/teahouse-relay /usr/lib/teahouse/teahouse-relay
COPY deploy/sshd/teahouse.conf /etc/ssh/sshd_config.d/teahouse.conf
COPY deploy/docker/entrypoint.sh /usr/local/bin/teahouse-entrypoint
RUN chmod 0755 /usr/lib/teahouse /usr/lib/teahouse/teahouse-relay /usr/local/bin/teahouse-entrypoint \
 && ln -s /usr/lib/teahouse/teahouse-relay /usr/local/bin/teahouse-relay \
 && printf 'HostKey /var/lib/teahouse/.sshd/ssh_host_ed25519_key\nPidFile /run/sshd.pid\n' > /etc/ssh/sshd_config.d/00-teahouse-hostkey.conf
ENV TEAHOUSE_ROOT=/var/lib/teahouse
VOLUME /var/lib/teahouse
EXPOSE 22
HEALTHCHECK --interval=5m --timeout=60s CMD runuser -u teahouse -- teahouse-relay check >/dev/null || exit 1
ENTRYPOINT ["/usr/local/bin/teahouse-entrypoint"]
