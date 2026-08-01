#!/usr/bin/env bash
set -euo pipefail
shopt -s nullglob

input_dir="${1:-.}"
output_dir="${2:-./sheets}"

if ! command -v magick >/dev/null 2>&1; then
    echo "오류: ImageMagick의 magick 명령이 필요합니다." >&2
    exit 1
fi

mkdir -p "$output_dir"

declare -A animated_groups=()
declare -A static_groups=()

png_files=("$input_dir"/*.png)

if ((${#png_files[@]} == 0)); then
    echo "오류: PNG 파일이 없습니다: $input_dir" >&2
    exit 1
fi

# 파일명 패턴별 group 검색
for path in "${png_files[@]}"; do
    filename="${path##*/}"

    # 예: attackerGunAttack1_0001.png
    if [[ "$filename" =~ ^(.+)([1-5])_([0-9]{4})[.]png$ ]]; then
        animated_groups["${BASH_REMATCH[1]}"]=1

    # 예: stand_0001.png
    elif [[ "$filename" =~ ^(.+)_([0-9]{4})[.]png$ ]]; then
        static_groups["${BASH_REMATCH[1]}"]=1
    fi
done

generated=0

# Animation sheet 생성
for prefix in "${!animated_groups[@]}"; do
    first_row=("$input_dir/${prefix}1_"[0-9][0-9][0-9][0-9].png)

    if ((${#first_row[@]} == 0)); then
        continue
    fi

    files=()
    frame_suffixes=()

    # 1번 방향의 frame 번호를 기준으로 삼음
    for path in "${first_row[@]}"; do
        filename="${path##*/}"
        suffix="${filename#"${prefix}1_"}"
        suffix="${suffix%.png}"
        frame_suffixes+=("$suffix")
    done

    for direction in {1..5}; do
        for suffix in "${frame_suffixes[@]}"; do
            file="$input_dir/${prefix}${direction}_${suffix}.png"

            if [[ ! -f "$file" ]]; then
                echo "오류: animation frame 누락: $file" >&2
                exit 1
            fi

            files+=("$file")
        done
    done

    frame_count="${#frame_suffixes[@]}"
    output="$output_dir/${prefix}_sheet.png"

    magick montage \
        "${files[@]}" \
        -tile "${frame_count}x5" \
        -geometry +0+0 \
        -background none \
        "$output"

    echo "Animation sheet 생성: $output (${frame_count}열 × 5행)"
    ((generated += 1))
done

# 단일 frame 방향 sheet 생성
for prefix in "${!static_groups[@]}"; do
    files=()

    for direction in {1..5}; do
        suffix="$(printf '%04d' "$direction")"
        file="$input_dir/${prefix}_${suffix}.png"

        if [[ ! -f "$file" ]]; then
            echo "오류: 방향 이미지 누락: $file" >&2
            exit 1
        fi

        files+=("$file")
    done

    output="$output_dir/${prefix}_sheet.png"

    magick montage \
        "${files[@]}" \
        -tile 1x5 \
        -geometry +0+0 \
        -background none \
        "$output"

    echo "방향 sheet 생성: $output (1열 × 5행)"
    ((generated += 1))
done

if ((generated == 0)); then
    echo "오류: 지원하는 파일명 패턴을 찾지 못했습니다." >&2
    exit 1
fi
