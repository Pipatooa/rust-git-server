FROM alpine:latest

RUN apk add --no-cache openssh git sed

RUN sed -i /etc/ssh/sshd_config \
    -e 's/#PermitRootLogin .\+/PermitRootLogin no/' \
    -e 's/#PasswordAuthentication \.+/PasswordAuthentication no/' \
    -e 's|#HostKey /etc/ssh/|HostKey /etc/ssh/keys/|'

RUN apk del sed
RUN mkdir /srv/data /etc/ssh/keys

WORKDIR /srv

COPY entrypoint.sh .

ENTRYPOINT ["./entrypoint.sh"]
CMD ["/usr/sbin/sshd", "-D"]
