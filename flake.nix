{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, ...  } @ inputs:
     let
      inherit self inputs;
      system = "x86_64-linux";

      overlays = [ (import rust-overlay) ];

      pkgs = import nixpkgs {
        inherit system overlays;

        config = {
          allowUnfree = true;
          android_sdk.accept_license = true;
        };
      };

      # androidPkgs = import nixpkgs {
      #   inherit system;
      #   crossSystem = {
      #     config = "aarch64-unknown-linux-android";
      #     rust.rustcTarget = "aarch64-linux-android";
      #
      #     sdkVersion = "36";
      #     ndkVersion = "27.2.12479018";
      #     # useAndroidPrebuilt = true;
      #   };
      #
      #   config = {
      #     allowUnfree = true;
      #     android_sdk.accept_license = true;
      #   };
      # };

      # make vergen_git2 happy
      VERGEN_IDEMPOTENT = "1";
      VERGEN_GIT_SHA = if (self ? "rev") then (builtins.substring 0 7 self.rev) else "nix-dirty";

      # NOTE copy these from QT docs
      # https://doc.qt.io/qt-6/android.html#supported-configurations
      androidVersion = {
        buildTools = "36.0.0";
        sdk = "36";
        ndk = "27.2.12479018";
      };

      androidPackages = pkgs.androidenv.composeAndroidPackages {
        includeNDK = true;
        buildToolsVersions = [ androidVersion.buildTools ];
        platformVersions = [ androidVersion.sdk ];
        ndkVersions = [ androidVersion.ndk ];
      };

      sdk = androidPackages.androidsdk;
      platformTools = androidPackages.platform-tools;

      rust = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
          "rustfmt"
          "clippy"
        ];
        targets = [
          "x86_64-unknown-linux-gnu"
          "aarch64-linux-android"
          "x86_64-linux-android"
        ];
      };

      qt6-android = pkgs.callPackage ./nix/qtbase.nix { inherit inputs; };
      qt6-env = with pkgs.qt6; env "qt6-host" [ qtbase qtdeclarative ];
    in
    rec {
      packages.${system} = {
        qt6-android = pkgs.callPackage ./nix/qt-for-android.nix {};
        # qt6-android = {
        #   qtbase = pkgs.callPackage ./nix/ {};
        # };
      };

      devShells.${system} = {
        default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            git
            ninja
            cmake
            gdb
            ccls # CXX LSP
            rust

            # # setting up the environment
            # qt6.wrapQtAppsHook
            # makeWrapper
            # bashInteractive

            wayland-scanner
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
            libx11
            libxscrnsaver
            libxcursor
            libxext
            libxfixes
            libxi
            libxrandr

            # androidPkgs.qt6.qtbase
            # pkgsCross.aarch64-android-prebuilt.qt6.qtbase
            # pkgsCross.aarch64-android-prebuilt.qt6.qtdeclarative
            # pkgsCross.aarch64-android-prebuilt.qt6.qttools

            # android
            sdk
            platformTools

            # androidPkgs.rustc
            # androidPkgs.cargo

            # androidPkgs.qt6.qtbase
            # androidPkgs.qt6.qtdeclarative
            # androidPkgs.qt6.qttools

            # androidPkgs.androidsdk
            # androidPkgs.androidndk

            javaPackages.compiler.openjdk17

            # qt6-android

            qtcreator

            # qt deps (all from qt6.full)
            # pkgsCross.aarch64-android-prebuilt.qt6.qtbase
            # pkgsCross.aarch64-android-prebuilt.qt6.qtquick3d

            # qt6.qtbase
            # qt6.qtdeclarative

            # qt6.qtlanguageserver
            # qt6.qtquick3d
            # qt6.qt3d
            # qt6.qt5compat
            # qt6.qtcharts
            # qt6.qtconnectivity
            # qt6.qtdatavis3d
            # qt6.qtdoc
            # qt6.qtgraphs
            # qt6.qtgrpc
            # qt6.qthttpserver
            # qt6.qtimageformats
            # qt6.qtlocation
            # qt6.qtlottie
            # qt6.qtmultimedia
            # qt6.qtmqtt
            # qt6.qtnetworkauth
            # qt6.qtpositioning
            # qt6.qtsensors
            # qt6.qtserialbus
            # qt6.qtserialport
            # qt6.qtshadertools
            # qt6.qtspeech
            # qt6.qtquick3dphysics
            # qt6.qtquickeffectmaker
            # qt6.qtquicktimeline
            # qt6.qtremoteobjects
            # qt6.qtsvg
            # qt6.qtscxml
            # qt6.qttools
            # qt6.qttranslations
            # qt6.qtvirtualkeyboard
            # qt6.qtwebchannel
            # qt6.qtwebengine
            # qt6.qtwebsockets
            # qt6.qtwebview
          ];

          # use Ninja generator by default
          CMAKE_GENERATOR = "Ninja";

          inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;

          ANDROID_SDK_ROOT = "${sdk}/libexec/android-sdk";
          ANDROID_HOME = "${sdk}/libexec/android-sdk";
          ANDROID_NDK_ROOT = "${sdk}/libexec/android-sdk/ndk/${androidVersion.ndk}";
          # GLESv2_INCLUDE_DIR = "${pkgs.libGL.dev}/include";
          # Vulkan_INCLUDE_DIR = "${pkgs.vulkan-headers}/include";
          # EGL_INCLUDE_DIR = "${pkgs.egl-wayland}/include";

          # QT_ANDROID = "${qt6-android}";
          # QT_HOST_PATH = "${qt6-env}";

          # shellHook = ''
          #   # export PATH="${ lib.makeLibraryPath [ pkgs.libGL ]}:$PATH"
          # '';
          # set the environment variables that unpatched Qt apps expect
          # shellHook = ''
          #   bashdir=$(mktemp -d)
          #   makeWrapper "$(type -p bash)" "$bashdir/bash" "''${qtWrapperArgs[@]}"
          #   exec "$bashdir/bash"
          # '';
        };
      };
    };
}
