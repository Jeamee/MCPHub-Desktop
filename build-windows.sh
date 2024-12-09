#!/bin/bash

# Exit on error
set -e

echo "🚀 Starting Windows build process..."

# Build frontend first
echo "📦 Building frontend..."
pnpm install
pnpm build

# Build the Docker image
echo "📦 Building Docker image..."
docker build -t windows-builder -f Dockerfile.windows .

# Create output directory if it doesn't exist
mkdir -p target/windows-build

# Run the build process
echo "🔨 Building Windows executable..."
docker run --rm \
    -v "$(pwd)":/app \
    -v "$(pwd)/target/windows-build":/app/src-tauri/target/x86_64-pc-windows-gnu/release \
    windows-builder \
    sh -c "cd src-tauri && TAURI_SKIP_DEVSERVER=true RUST_BACKTRACE=1 cargo tauri build --target x86_64-pc-windows-gnu --verbose"

echo "✅ Build complete! Check target/windows-build for your Windows executable."