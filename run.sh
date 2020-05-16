#!/usr/bin/env bash

if [ "$1" = "rebuild" ]; then
    docker build --tag=juniper-demo -f ./Dockerfile .
fi

docker-compose up -d postgres
docker-compose up -d graphql