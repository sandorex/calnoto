{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, fenix, ...  }:
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

      # make vergen_git2 happy
      VERGEN_IDEMPOTENT = "1";
      VERGEN_GIT_SHA = if (self ? "rev") then (builtins.substring 0 7 self.rev) else "nix-dirty";

      # copy all of these when copying from godot config.gradle
      # https://github.com/godotengine/godot/blob/4.5/platform/android/java/app/config.gradle
      androidVersion = {
        buildTools = "35.0.1";
        sdk = "35";
        ndk = "28.1.13356709";
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

            gdb
            qt6.wrapQtAppsHook
            makeWrapper
            bashInteractive
            ninja
            just
            cmake
            validatePkgConfig
            # gcc
            wayland-scanner
            # zenity
            # libffi
            # python313
            # patchelf

            # TODO temp equivalent to qt6.full
            qt6.qtbase
            qt6.qt3d
            qt6.qt5compat
            qt6.qtcharts
            qt6.qtconnectivity
            qt6.qtdatavis3d
            qt6.qtdeclarative
            qt6.qtdoc
            qt6.qtgraphs
            qt6.qtgrpc
            qt6.qthttpserver
            qt6.qtimageformats
            qt6.qtlanguageserver
            qt6.qtlocation
            qt6.qtlottie
            qt6.qtmultimedia
            qt6.qtmqtt
            qt6.qtnetworkauth
            qt6.qtpositioning
            qt6.qtsensors
            qt6.qtserialbus
            qt6.qtserialport
            qt6.qtshadertools
            qt6.qtspeech
            qt6.qtquick3d
            qt6.qtquick3dphysics
            qt6.qtquickeffectmaker
            qt6.qtquicktimeline
            qt6.qtremoteobjects
            qt6.qtsvg
            qt6.qtscxml
            qt6.qttools
            qt6.qttranslations
            qt6.qtvirtualkeyboard
            qt6.qtwebchannel
            qt6.qtwebengine
            qt6.qtwebsockets
            qt6.qtwebview
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

          inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;

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
