{
  description = "Vel: A web templating library inspired by Svelte";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    zig2nix.url = "github:Cloudef/zig2nix";
    nvame.url = "github:namescode/nvame";
  };

  nixConfig = {
    extra-substituters = [ "https://zig2nix.cachix.org" ];
  };

  outputs =
    {
      self,
      nixpkgs,
      nvame,
      zig2nix,
    }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              (nvame.packages.${system}.mainConfig)

              # Rust tooling
              rustc
              rustfmt
              cargo
              clippy

              # Zig tooling
              (zig2nix.packages.${system}.zig-latest)
              # zig

              # Linter
              reuse

              # LSPs
              rust-analyzer
              zls
              emmet-language-server
              nodePackages.vscode-langservers-extracted
            ];

            shellHook = ''echo "You have now entered the dev shell for Zest, exit at any time."'';
          };
        }
      );
    };
}
