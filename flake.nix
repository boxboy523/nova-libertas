{
  description = "Rust + Bevy Development Environment on NixOS";

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

        cargoBuilder = pkgs.writeShellScriptBin "b" ''
          cargo build "$@"
        '';

        # Bevy 빌드 및 런타임에 필요한 라이브러리들
        buildInputs = with pkgs; [
          # 빌드 도구
          pkg-config
          alsa-lib
          udev
          openssl
          llvmPackages.libclang
          clang
          fontconfig
          freetype

          cargoBuilder

          # 그래픽스 및 윈도우 시스템 (Vulkan, Wayland/X11)
          vulkan-loader
          vulkan-validation-layers
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
          #VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          VK_ICD_FILENAMES = "/run/opengl-driver/share/vulkan/icd.d/radeon_icd.x86_64.json";

          shellHook = ''
            echo "Cargo Version: $(cargo --version)"
          '';
        };
      }
    );
}
