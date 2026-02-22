#!/usr/bin/env bash
set -euo pipefail

# Base Color
ktx create \
  --format R8G8B8A8_SRGB \
  --assign-tf srgb \
  --layers 2 \
  assets/textures/raw/brick_basecolor.png \
  assets/textures/raw/render_basecolor.png \
  assets/temp/uncompressed_base_color.ktx2

kram encode \
  -f bc7 \
  -type 2darray \
  -srgb \
  -zstd 0 \
  -o assets/textures/base_color.ktx2 \
  -i assets/temp/uncompressed_base_color.ktx2

# Normals
ktx create \
  --format R8G8_UNORM \
  --assign-tf linear \
  --normal-mode \
  --layers 2 \
  assets/textures/raw/brick_normal.png \
  assets/textures/raw/render_normal.png \
  assets/temp/normal.ktx2

kram encode \
  -f bc5 \
  -type 2darray \
  -normal \
  -o assets/textures/normal.ktx2 \
  -i assets/temp/normal.ktx2

# Alternative for keeping 16 bit precision (didn't figure out how to compress this one tho)
# ktx create \
#   --format R16G16_UNORM \
#   --assign-tf linear \
#   --normal-mode \
#   --generate-mipmap \
#   --layers 2 \
#   assets/textures/raw/brick_normal.png \
#   assets/textures/raw/render_normal.png \
#   assets/textures/normal.ktx2

# Linear
ktx create \
  --format R8G8B8_UNORM \
  --assign-tf linear \
  --layers 2 \
  assets/textures/raw/brick_arm.png \
  assets/textures/raw/render_arm.png \
  assets/temp/arm.ktx2
kram encode \
  -f bc7 \
  -type 2darray \
  -srclin  \
  -o assets/textures/arm.ktx2 \
  -i assets/temp/arm.ktx2

ktx create \
  --format R8_UNORM \
  --assign-tf linear \
  assets/textures/raw/wear_mask.png \
  assets/temp/wear_mask.ktx2
kram encode \
  -f bc4 \
  -type 2darray \
  -srclin  \
  -o assets/textures/wear_mask.ktx2 \
  -i assets/temp/wear_mask.ktx2

rm -r assets/temp/
