FROM alpine:latest

RUN apk add --no-cache openssh git

RUN sed -i /etc/ssh/sshd_config \
    -e 's/#PermitRootLogin .\+/PermitRootLogin no/' \
    -e 's/#PasswordAuthentication .\+/PasswordAuthentication no/' \
    -e 's|#HostKey /etc/ssh/|HostKey /etc/ssh/keys/|'

RUN mkdir /srv/bin /srv/data /etc/ssh/keys /root/git-shell-commands

RUN mkdir /etc/skel /etc/skel/.ssh /etc/skel/git-shell-commands  \
    && touch /etc/skel/.ssh/authorized_keys  \
    && chmod 700 /etc/skel/.ssh  \
    && chmod 600 /etc/skel/.ssh/authorized_keys

WORKDIR /srv
ENV PATH "$PATH:/srv/bin"

RUN ln -s /usr/bin/git-shell bin/manage

COPY entrypoint.sh .
COPY manage /root/git-shell-commands
COPY commands /etc/skel/git-shell-commands

ENTRYPOINT ["./entrypoint.sh"]
CMD ["/usr/sbin/sshd", "-D"]
