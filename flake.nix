{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
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

      # NOTE this the godot version used to build the application
      godot = pkgs.godotPackages_4_5;

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
            godot.godot
            godot.export-templates-bin

            cargo
            rust-analyzer
            clippy
            rustfmt
            rustc
          ];

          inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
        };

        android =
          let
            androidComposition = pkgs.androidenv.composeAndroidPackages {
              buildToolsVersions = [ androidVersion.buildTools ];
              platformVersions = [ androidVersion.sdk ];
              ndkVersions = [ androidVersion.ndk ];
              # platformToolsVersion = ""; # NOTE not in config.gradle??
              includeNDK = true;

              # TODO idk if its needed
              # includeExtras = [ "extras;google;auto" ];
            };

            pkgs' = pkgs.pkgsCross.aarch64-android-prebuilt;
          in
          pkgs'.mkShell rec {
            nativeBuildInputs = with pkgs; [
              git
              # TODO godot does not find the templates
              godot.godot
              godot.export-templates

              (with fenix.packages.${system}; combine [
                stable.toolchain
                targets.aarch64-linux-android.stable.rust-std
                targets.x86_64-linux-android.stable.rust-std # for emulator
              ])

              androidComposition.androidsdk
              javaPackages.compiler.openjdk17

              # cc crate looks for `cc` executable instead of $CC env var and so fails to build
              (pkgs.linkFarm "gcc-link" [
                {
                  name = "bin/cc";
                  path = "${pkgs'.stdenv.cc}/bin/${pkgs'.stdenv.cc.targetPrefix}cc";
                }
              ])
            ];

            ANDROID_HOME = "${androidComposition.androidsdk}/libexec/android-sdk";
            ANDROID_NDK_ROOT = "${ANDROID_HOME}/ndk-bundle";
            GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${ANDROID_HOME}/build-tools/${androidVersion.buildTools}/aapt2";

            CARGO_BUILD_TARGET = "aarch64-linux-android";

            inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
          };
      };
    };
}
