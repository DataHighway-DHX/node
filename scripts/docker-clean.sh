#!/bin/bash
# Stop all running containers
docker stop $(docker ps -aq)
# Delete all containers. Similar to `docker-compose down --rmi all`
docker rm $(docker ps -aq)
# Delete all images
docker rmi $(docker images -q)
# Delete cache
docker system prune -a -f
