#!/bin/bash

# Docker push script for aipriceaction-proxy
# Usage: ./scripts/docker-push.sh <tag>
# Example: ./scripts/docker-push.sh v1.0.0

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check for help flag
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    echo "Docker push script for aipriceaction-proxy"
    echo ""
    echo "Usage: $0 <tag>"
    echo ""
    echo "Arguments:"
    echo "  <tag>    Version tag to push (must match previously built tag)"
    echo ""
    echo "Environment Variables:"
    echo "  DOCKERHUB_USERNAME    Your DockerHub username (required)"
    echo ""
    echo "Prerequisites:"
    echo "  - Images built with docker-build-and-tag.sh"
    echo "  - Logged in to DockerHub (docker login)"
    echo ""
    echo "Examples:"
    echo "  export DOCKERHUB_USERNAME=myuser"
    echo "  docker login"
    echo "  $0 v1.0.0"
    echo ""
    echo "For more information, see scripts/README.md"
    exit 0
fi

# Check if tag is provided
if [ $# -eq 0 ]; then
    print_error "No tag provided!"
    echo "Usage: $0 <tag>"
    echo "Example: $0 v1.0.0"
    echo "Run '$0 --help' for more information."
    exit 1
fi

TAG="$1"
IMAGE_NAME="aipriceaction-proxy"
DOCKERHUB_USERNAME="${DOCKERHUB_USERNAME:-}"

# Check if DOCKERHUB_USERNAME is set
if [ -z "$DOCKERHUB_USERNAME" ]; then
    print_error "DOCKERHUB_USERNAME environment variable is not set!"
    print_info "Set it with: export DOCKERHUB_USERNAME=your-dockerhub-username"
    exit 1
fi

print_info "Pushing Docker images to DockerHub..."
print_info "Username: $DOCKERHUB_USERNAME"
print_info "Tag: $TAG"

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    print_error "Docker is not running or not accessible!"
    print_info "Please start Docker and try again."
    exit 1
fi

# Check if user is logged in to DockerHub
if ! docker info | grep -q "Username"; then
    print_warning "You may not be logged in to DockerHub."
    print_info "Run 'docker login' if push fails."
fi

# Get git commit for additional tag
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Check if images exist locally
IMAGES_TO_PUSH=(
    "$DOCKERHUB_USERNAME/$IMAGE_NAME:$TAG"
    "$DOCKERHUB_USERNAME/$IMAGE_NAME:latest"
    "$DOCKERHUB_USERNAME/$IMAGE_NAME:$GIT_COMMIT"
)

print_info "Checking for local images..."
for image in "${IMAGES_TO_PUSH[@]}"; do
    if docker image inspect "$image" >/dev/null 2>&1; then
        print_success "Found: $image"
    else
        print_warning "Not found: $image"
        print_info "Run ./scripts/docker-build-and-tag.sh $TAG first"
    fi
done

print_info "Starting push process..."

# Push each image
for image in "${IMAGES_TO_PUSH[@]}"; do
    if docker image inspect "$image" >/dev/null 2>&1; then
        print_info "Pushing $image..."
        if docker push "$image"; then
            print_success "Successfully pushed: $image"
        else
            print_error "Failed to push: $image"
            exit 1
        fi
    else
        print_warning "Skipping $image (not found locally)"
    fi
done

print_success "All available images pushed successfully!"

# Show DockerHub URLs
print_info "Your images are now available at:"
echo "  https://hub.docker.com/r/$DOCKERHUB_USERNAME/$IMAGE_NAME"
echo ""
print_info "To pull the images:"
echo "  docker pull $DOCKERHUB_USERNAME/$IMAGE_NAME:$TAG"
echo "  docker pull $DOCKERHUB_USERNAME/$IMAGE_NAME:latest"
if [ "$GIT_COMMIT" != "unknown" ]; then
    echo "  docker pull $DOCKERHUB_USERNAME/$IMAGE_NAME:$GIT_COMMIT"
fi