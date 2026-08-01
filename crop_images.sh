#!/usr/bin/env bash
set -euo pipefail

input_dir="${1:?usage: crop_images.sh INPUT_DIR OUTPUT_DIR [SIZE]}"
output_dir="${2:?usage: crop_images.sh INPUT_DIR OUTPUT_DIR [SIZE]}"
crop_size="${3:-256}"

if ! command -v magick >/dev/null 2>&1; then
    echo "ImageMagick의 magick 명령을 찾을 수 없습니다." >&2
    exit 1
fi

mkdir -p "$output_dir"

find "$input_dir" -type f -iname '*.png' -print0 |
while IFS= read -r -d '' source_path; do
    relative_path="${source_path#"$input_dir"/}"
    output_path="$output_dir/$relative_path"

    mkdir -p "$(dirname "$output_path")"

    magick "$source_path" \
        -gravity center \
        -crop "${crop_size}x${crop_size}+0+0" \
        +repage \
        "$output_path"

    echo "$source_path -> $output_path"
done
