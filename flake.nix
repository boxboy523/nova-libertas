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

        # Bevy 빌드 및 런타임에 필요한 라이브러리들
        buildInputs = with pkgs; [
          # 빌드 도구
          alsa-lib
          udev
          openssl
          fontconfig
          freetype

          # 그래픽스 및 윈도우 시스템 (Vulkan, Wayland/X11)
          libxkbcommon
          xkeyboard-config
          wayland
          libx11
          libxcursor
          libxrandr
          libxi

          vulkan-loader
          mesa
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs;

          nativeBuildInputs = [
            rustToolchain
            pkgs.pkg-config
            pkgs.clang
            pkgs.llvmPackages.libclang
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.alsa-lib
            pkgs.udev
            pkgs.libxkbcommon
            pkgs.wayland

            pkgs.vulkan-loader
            pkgs.mesa
          ];


          XKB_CONFIG_ROOT = "${pkgs.xkeyboard-config}/share/X11/xkb";
          VK_DRIVER_FILES =
            "${pkgs.mesa}/share/vulkan/icd.d/radeon_icd.x86_64.json";
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          shellHook = ''
            echo "Cargo Version: $(cargo --version)"
          '';
        };
      }
    );
}
