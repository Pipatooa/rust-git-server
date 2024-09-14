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

RUN for alias in init add;      do ln -s /srv/commands/create /srv/commands/$alias; done && \
    for alias in rm remove del; do ln -s /srv/commands/delete /srv/commands/$alias; done && \
    for alias in mv;            do ln -s /srv/commands/move   /srv/commands/$alias; done && \
    for alias in ls l dir;      do ln -s /srv/commands/list   /srv/commands/$alias; done

WORKDIR /srv
ENV PATH "$PATH:/srv/bin"

RUN ln -s /usr/bin/git-shell bin/manage

COPY entrypoint.sh .
COPY manage /root/git-shell-commands
COPY commands /srv/commands

ENTRYPOINT ["./entrypoint.sh"]
CMD ["/usr/sbin/sshd", "-D"]
