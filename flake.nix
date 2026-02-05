{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, ...  }:
    let
      inherit self;
      system = "x86_64-linux";

      pkgs = import nixpkgs { inherit system; };

      # make vergen_git2 happy
      VERGEN_IDEMPOTENT = "1";
      VERGEN_GIT_SHA = if (self ? "rev") then (builtins.substring 0 7 self.rev) else "nix-dirty";

      # NOTE this determines the minimal supported godot version
      godot = pkgs.godotPackages_4_5;
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

            godot.godot
            godot.export-templates-bin
          ];

          inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
        };

        android = pkgs.mkShell {
          inputsFrom = [ desktop ];

          # TODO add android sdk and adb
          nativeBuildInputs = with pkgs; [
            javaPackages.compiler.openjdk17
          ];

          inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
        };
      };
    };
}
