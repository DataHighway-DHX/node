#!/bin/bash
# Stop all running containers
docker stop $(docker ps -aq)
# Delete all containers
docker rm $(docker ps -aq)
# Delete all images
docker rmi $(docker images -q)
