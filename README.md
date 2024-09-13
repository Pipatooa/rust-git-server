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
      - ./data:/srv/data:Z
      - ./keys:/etc/ssh/keys:Z
```
