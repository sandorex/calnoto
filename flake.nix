{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
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
        buildTools = "35.0.0";
        sdk = "36";
        ndk = "28.2.13676358";
        cmake = "3.22.1";
      };

      androidComposition = pkgs.androidenv.composeAndroidPackages {
        buildToolsVersions = [ androidVersion.buildTools ];
        platformVersions = [ androidVersion.sdk ];
        ndkVersions = [ androidVersion.ndk ];
        includeNDK = true;
        cmakeVersions = [ androidVersion.cmake ];
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
          ];

          inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
        };

        android = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            git
            flutter
            androidComposition.androidsdk
            androidComposition.platform-tools
            androidComposition.cmake
            javaPackages.compiler.openjdk17
          ];

          ANDROID_HOME = "${androidComposition.androidsdk}/libexec/android-sdk";
          ANDROID_SDK_ROOT = "${androidComposition.androidsdk}/libexec/android-sdk";
          ANDROID_NDK_ROOT = "${ANDROID_HOME}/ndk-bundle";
          GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${ANDROID_HOME}/build-tools/${androidVersion.buildTools}/aapt2";

          CARGO_BUILD_TARGET = "aarch64-linux-android";

          inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
        };
      };
    };
}
