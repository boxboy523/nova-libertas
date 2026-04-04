{
  description = "Rust + Godot 4 Development Environment on NixOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Rust 최신 안정 버전 (Rust-analyzer 포함)
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        godotLauncher = pkgs.writeShellScriptBin "g" ''
          PROJ_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
          export PATH="${pkgs.jq}/bin:$PATH"

          TERM_INFO=$(hyprctl activewindow -j)
          TERM_ADDR=$(echo "$TERM_INFO" | jq -r '.address')
          ORIGIN_WS=$(echo "$TERM_INFO" | jq -r '.workspace.name')

          # 터미널 숨기기 (Special Workspace로 이동)
          hyprctl dispatch movetoworkspacesilent "special:hidden,address:$TERM_ADDR"

          # Godot 에디터 실행 (백그라운드가 아니라 포그라운드에서 실행해 종료 대기)
          godot4 -e --path "$PROJ_ROOT/godot"

          # Godot 종료 후 터미널 복구
          hyprctl dispatch movetoworkspace "$ORIGIN_WS,address:$TERM_ADDR"
          hyprctl dispatch focuswindow "address:$TERM_ADDR"
        '';

        cargoBuilder = pkgs.writeShellScriptBin "b" ''
        CURRENT_DIR=$(pwd)
        PROJ_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)

        cd $PROJ_ROOT/rust
        cargo build "$@"

        LIB_SRC="$PROJ_ROOT/rust/target/debug/libgame_backend.so"
        LIB_DST="$PROJ_ROOT/godot/bin/libgame_backend.so"
        if [ -f "$LIB_SRC" ]; then
          mkdir -p "$(dirname "$LIB_DST")"

          if [ ! -L "$LIB_DST" ] || [ "$(readlink -f "$LIB_DST")" != "$LIB_SRC" ]; then
            ln -sf "$LIB_SRC" "$LIB_DST"
            echo "Updated symlink: $LIB_DST -> $LIB_SRC"
          else
            echo "Symlink already up to date: $LIB_DST -> $LIB_SRC"
          fi
        fi
        cd "$CURRENT_DIR"
        '';

        # Godot 및 런타임에 필요한 라이브러리들
        buildInputs = with pkgs; [
          godot_4          # Godot 4 에디터

          # 빌드 도구
          pkg-config
          openssl
          jq
          llvmPackages.libclang
          clang
          fontconfig
          freetype

          godotLauncher
          cargoBuilder

          # 그래픽스 및 윈도우 시스템 (Vulkan, Wayland/X11)
          vulkan-loader
          libxkbcommon
          wayland
          libx11
          libxcursor
          libxrandr
          libxi
          libglvnd
          libGL
          vulkan-headers
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs;

          nativeBuildInputs = [ rustToolchain ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          shellHook = ''
            echo "Godot + Rust DevShell Activated!"
            echo "Godot Version: $(godot4 --version)"
            echo "Cargo Version: $(cargo --version)"
          '';
        };
      }
    );
}
