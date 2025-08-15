#!/bin/bash

# Docker build and tag script for aipriceaction-proxy
# Usage: ./scripts/docker-build-and-tag.sh <tag>
# Example: ./scripts/docker-build-and-tag.sh v1.0.0

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
    echo "Docker build and tag script for aipriceaction-proxy"
    echo ""
    echo "Usage: $0 <tag>"
    echo ""
    echo "Arguments:"
    echo "  <tag>    Version tag for the Docker image (e.g., v1.0.0, latest, dev)"
    echo ""
    echo "Environment Variables:"
    echo "  DOCKERHUB_USERNAME    Your DockerHub username (optional)"
    echo ""
    echo "Examples:"
    echo "  $0 v1.0.0                           # Build and tag locally"
    echo "  DOCKERHUB_USERNAME=user $0 v1.0.0   # Build and tag for DockerHub"
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

# Validate tag format (basic validation)
if [[ ! "$TAG" =~ ^[a-zA-Z0-9._-]+$ ]]; then
    print_error "Invalid tag format: $TAG"
    print_info "Tag should contain only alphanumeric characters, dots, hyphens, and underscores"
    exit 1
fi

print_info "Starting Docker build process for tag: $TAG"
print_info "Image name: $IMAGE_NAME"

# Check if we're in the right directory (should contain Dockerfile)
if [ ! -f "Dockerfile" ]; then
    print_error "Dockerfile not found in current directory!"
    print_info "Please run this script from the project root directory."
    exit 1
fi

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    print_error "Docker is not running or not accessible!"
    print_info "Please start Docker and try again."
    exit 1
fi

# Get git commit hash for additional tagging
GIT_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

print_info "Git commit: $GIT_COMMIT"
print_info "Build date: $BUILD_DATE"

# Build the Docker image
print_info "Building Docker image..."
docker build \
    --build-arg BUILD_DATE="$BUILD_DATE" \
    --build-arg GIT_COMMIT="$GIT_COMMIT" \
    --tag "$IMAGE_NAME:$TAG" \
    --tag "$IMAGE_NAME:latest" \
    .

if [ $? -eq 0 ]; then
    print_success "Docker image built successfully!"
else
    print_error "Docker build failed!"
    exit 1
fi

# Tag for DockerHub if username is provided
if [ -n "$DOCKERHUB_USERNAME" ]; then
    print_info "Tagging for DockerHub with username: $DOCKERHUB_USERNAME"
    
    # Tag with username for DockerHub
    docker tag "$IMAGE_NAME:$TAG" "$DOCKERHUB_USERNAME/$IMAGE_NAME:$TAG"
    docker tag "$IMAGE_NAME:$TAG" "$DOCKERHUB_USERNAME/$IMAGE_NAME:latest"
    
    # Also tag with git commit
    docker tag "$IMAGE_NAME:$TAG" "$DOCKERHUB_USERNAME/$IMAGE_NAME:$GIT_COMMIT"
    
    print_success "Tagged images for DockerHub:"
    echo "  - $DOCKERHUB_USERNAME/$IMAGE_NAME:$TAG"
    echo "  - $DOCKERHUB_USERNAME/$IMAGE_NAME:latest"
    echo "  - $DOCKERHUB_USERNAME/$IMAGE_NAME:$GIT_COMMIT"
    
    print_info "To push to DockerHub, run:"
    echo "  docker push $DOCKERHUB_USERNAME/$IMAGE_NAME:$TAG"
    echo "  docker push $DOCKERHUB_USERNAME/$IMAGE_NAME:latest"
    echo "  docker push $DOCKERHUB_USERNAME/$IMAGE_NAME:$GIT_COMMIT"
    
    print_warning "Make sure you're logged in to DockerHub: docker login"
else
    print_warning "DOCKERHUB_USERNAME environment variable not set."
    print_info "Set it to enable DockerHub tagging:"
    echo "  export DOCKERHUB_USERNAME=your-dockerhub-username"
    echo "  $0 $TAG"
fi

# Show final image information
print_info "Built images:"
docker images | grep "$IMAGE_NAME" | head -10

# Optional: Test the built image
print_info "Testing the built image..."
if docker run --rm "$IMAGE_NAME:$TAG" --help >/dev/null 2>&1; then
    print_success "Image test passed!"
else
    print_warning "Image test failed or binary doesn't support --help flag"
    print_info "You may want to test the image manually:"
    echo "  docker run --rm -p 8888:8888 $IMAGE_NAME:$TAG"
fi

print_success "Docker build and tag process completed!"
print_info "Local tags created:"
echo "  - $IMAGE_NAME:$TAG"
echo "  - $IMAGE_NAME:latest"

if [ -n "$DOCKERHUB_USERNAME" ]; then
    echo ""
    print_info "DockerHub ready tags:"
    echo "  - $DOCKERHUB_USERNAME/$IMAGE_NAME:$TAG"
    echo "  - $DOCKERHUB_USERNAME/$IMAGE_NAME:latest" 
    echo "  - $DOCKERHUB_USERNAME/$IMAGE_NAME:$GIT_COMMIT"
fi