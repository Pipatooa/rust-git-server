# Docker Local Git Server

A simple local git server for Docker

### Example `docker-compose.yml`:
```yml
services:
  git:
    build: .
    restart: unless-stopped
    ports:
      - "4022:22"
    volumes:
      - ./keys:/etc/ssh/keys:Z
      - ./repos:/srv/repos:Z
      - ./users:/srv/users:Z
```

To manage the git server, use `docker exec <container> manage`.
