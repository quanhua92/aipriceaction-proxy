# Docker Build and Push Scripts

This directory contains scripts for building and pushing Docker images to DockerHub.

## Scripts

### 1. `docker-build-and-tag.sh`

Builds a Docker image and tags it for local use and DockerHub.

**Usage:**
```bash
./scripts/docker-build-and-tag.sh <tag>
```

**Example:**
```bash
# Set your DockerHub username (optional but recommended)
export DOCKERHUB_USERNAME=your-dockerhub-username

# Build and tag the image
./scripts/docker-build-and-tag.sh v1.0.0
```

**What it does:**
- Builds the Docker image using the multi-stage Dockerfile
- Tags the image with the provided tag and `latest`
- If `DOCKERHUB_USERNAME` is set, creates additional tags for DockerHub
- Includes git commit hash and build date metadata
- Tests the built image
- Shows push commands for DockerHub

**Tags created:**
- `aipriceaction-proxy:<tag>`
- `aipriceaction-proxy:latest`
- `<username>/aipriceaction-proxy:<tag>` (if username set)
- `<username>/aipriceaction-proxy:latest` (if username set)
- `<username>/aipriceaction-proxy:<git-commit>` (if username set)

### 2. `docker-push.sh`

Pushes pre-built Docker images to DockerHub.

**Usage:**
```bash
./scripts/docker-push.sh <tag>
```

**Example:**
```bash
# Set your DockerHub username (required)
export DOCKERHUB_USERNAME=your-dockerhub-username

# Login to DockerHub
docker login

# Push the images
./scripts/docker-push.sh v1.0.0
```

**Prerequisites:**
- Images must be built first using `docker-build-and-tag.sh`
- Must be logged in to DockerHub (`docker login`)
- `DOCKERHUB_USERNAME` environment variable must be set

## Complete Workflow

### 1. First-time setup
```bash
# Set your DockerHub username
export DOCKERHUB_USERNAME=your-dockerhub-username

# Login to DockerHub
docker login
```

### 2. Build and push a release
```bash
# Build and tag the image
./scripts/docker-build-and-tag.sh v1.2.3

# Push to DockerHub
./scripts/docker-push.sh v1.2.3
```

### 3. Quick development build
```bash
# Build without DockerHub tagging
./scripts/docker-build-and-tag.sh dev-$(git rev-parse --short HEAD)
```

## Environment Variables

- `DOCKERHUB_USERNAME`: Your DockerHub username (required for DockerHub operations)

## Features

- **Multi-architecture support**: Uses Alpine Linux for minimal image size
- **Security**: Runs as non-root user
- **Caching**: Uses cargo-chef for fast incremental builds
- **Metadata**: Includes git commit hash and build date
- **Validation**: Checks for required tools and prerequisites
- **Testing**: Basic smoke test of built images
- **Colored output**: Easy-to-read console output

## Image Information

- **Base image**: Alpine Linux 3.22
- **Architecture**: Native musl target
- **User**: Non-root (`appuser`)
- **Port**: 8080 (configurable)
- **Size**: Optimized for minimal footprint

## Troubleshooting

### Docker not running
```bash
# Start Docker Desktop or Docker daemon
# Then retry the script
```

### Not logged in to DockerHub
```bash
docker login
# Enter your DockerHub credentials
```

### Permission denied
```bash
chmod +x scripts/docker-build-and-tag.sh
chmod +x scripts/docker-push.sh
```

### Build failures
- Ensure you're in the project root directory
- Check that all source files are present
- Verify Dockerfile exists
- Check for syntax errors in Rust code

## Examples

### Release workflow
```bash
# Set environment
export DOCKERHUB_USERNAME=mycompany

# Login
docker login

# Build and push release
./scripts/docker-build-and-tag.sh v2.1.0
./scripts/docker-push.sh v2.1.0

# Your image is now available as:
# mycompany/aipriceaction-proxy:v2.1.0
# mycompany/aipriceaction-proxy:latest
```

### Development workflow  
```bash
# Quick local build for testing
./scripts/docker-build-and-tag.sh dev-local

# Test locally
docker run --rm -p 8888:8888 aipriceaction-proxy:dev-local
```