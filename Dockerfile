FROM alpine:latest

RUN apk add --no-cache openssh git

RUN sed -i /etc/ssh/sshd_config \
    -e 's/#PermitRootLogin .\+/PermitRootLogin no/' \
    -e 's/#PasswordAuthentication .\+/PasswordAuthentication no/' \
    -e 's|#HostKey /etc/ssh/|HostKey /etc/ssh/keys/|' \
    && echo -n "" > /etc/motd

RUN mkdir /srv/bin /srv/commands /srv/data /etc/ssh/keys /root/git-shell-commands

RUN mkdir /etc/skel /etc/skel/.ssh  \
    && touch /etc/skel/.ssh/authorized_keys  \
    && chmod 700 /etc/skel/.ssh  \
    && chmod 644 /etc/skel/.ssh/authorized_keys \
    && ln -s /srv/commands /etc/skel/git-shell-commands

WORKDIR /srv
ENV PATH "$PATH:/srv/bin"

RUN ln -s /usr/bin/git-shell bin/manage

COPY entrypoint.sh .
COPY manage /root/git-shell-commands
COPY commands /srv/commands

ENTRYPOINT ["./entrypoint.sh"]
CMD ["/usr/sbin/sshd", "-D"]
