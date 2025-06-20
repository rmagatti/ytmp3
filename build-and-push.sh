#!/bin/bash

# Build and push Docker image to GitHub Container Registry
# Usage: ./build-and-push.sh [your-github-username]

GITHUB_USERNAME=${1:-rmagatti}
IMAGE_NAME="ghcr.io/${GITHUB_USERNAME}/ytmp3"

echo "Building Docker image: ${IMAGE_NAME}"

# Build the image
docker build -t "${IMAGE_NAME}:latest" .

if [ $? -eq 0 ]; then
    echo "Build successful! Pushing to GitHub Container Registry..."
    
    # Tag with git commit hash as well
    GIT_HASH=$(git rev-parse --short HEAD)
    docker tag "${IMAGE_NAME}:latest" "${IMAGE_NAME}:${GIT_HASH}"
    
    # Push both tags
    docker push "${IMAGE_NAME}:latest"
    docker push "${IMAGE_NAME}:${GIT_HASH}"
    
    echo "Successfully pushed:"
    echo "  ${IMAGE_NAME}:latest"
    echo "  ${IMAGE_NAME}:${GIT_HASH}"
else
    echo "Build failed!"
    exit 1
fi