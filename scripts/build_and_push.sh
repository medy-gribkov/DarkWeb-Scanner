#!/usr/bin/env bash
set -euo pipefail
IMAGE_NAME=${IMAGE_NAME:-sporesec-darkweb-scanner}
TAG=${TAG:-latest}

docker build -t ${IMAGE_NAME}:${TAG} .
# optionally push
# docker push ${IMAGE_NAME}:${TAG}
echo "Built ${IMAGE_NAME}:${TAG}"
