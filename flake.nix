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

      devTools = with pkgs; [
        # LSPs
        vscode-langservers-extracted
        typescript-language-server
        rust-analyzer

        # rust dev stuff
        rust-analyzer
        clippy
        rustfmt
      ];
    in
    {
      devShells.${system} = rec {
        default = desktop;

        desktop-build = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            git
            cargo
            rustc
            nodejs_24
            pkg-config
            gobject-introspection
          ];

          buildInputs = with pkgs; [
            at-spi2-atk
            atkmm
            cairo
            gdk-pixbuf
            glib
            gtk3
            harfbuzz
            librsvg
            libsoup_3
            pango
            webkitgtk_4_1
            openssl
          ];

          inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
        };

        desktop = pkgs.mkShell {
          inputsFrom = [ desktop-build ];

          nativeBuildInputs = devTools;

          inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
        };

        # TODO
        android-build = pkgs.mkShell {
          nativeBuildInputs = with pkgs []; [];

          inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
        };

        android = pkgs.mkShell {
          inputsFrom = [ android-build ];

          nativeBuildInputs = devTools;

          inherit VERGEN_IDEMPOTENT VERGEN_GIT_SHA;
        };
      };
    };
}
