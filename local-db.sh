!#/usr/bin/env bash
docker run --rm -it -e POSTGRES_PASSWORD=yourpassword -p 5432:5432 postgres:alpine