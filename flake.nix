{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs  }:
    let
      inherit self;
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;

        config = {
          allowUnfree = true;
          android_sdk.accept_license = true;
        };
      };
    in
    {
      devShells.${system} = rec {
        default = desktop;

        desktop = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            git

            cargo
            rust-analyzer
            clippy
            rustfmt
            rustc
            ccls

            gdb
            makeWrapper
            bashInteractive
            ninja
            just
            cmake

            validatePkgConfig
            wayland-scanner

            qt6.wrapQtAppsHook
            qt6.qtbase
            qt6.qtdeclarative
          ];

          # runtime dependencies
          buildInputs = with pkgs; [
            vulkan-headers
            vulkan-loader
            libGL
            libusb1
            libayatana-appindicator
            libdrm
            mesa
            wayland
            wayland-protocols
            pipewire
            libpulseaudio
            alsa-lib
            dbus
            libxkbcommon
            xorg.libX11
            xorg.libXScrnSaver
            xorg.libXcursor
            xorg.libXext
            xorg.libXfixes
            xorg.libXi
            xorg.libXrandr
          ];

          # use Ninja generator by default
          CMAKE_GENERATOR = "Ninja";

          # set the environment variables that unpatched Qt apps expect
          shellHook = ''
            bashdir=$(mktemp -d)
            makeWrapper "$(type -p bash)" "$bashdir/bash" "''${qtWrapperArgs[@]}"
            exec "$bashdir/bash"
          '';
        };
      };
    };
}
