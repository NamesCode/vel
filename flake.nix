{
  description = "Vel: A web templating library inspired by Svelte";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs =
    {
      self,
      nixpkgs,
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
              # Rust tooling
              rustc
              rustfmt
              cargo
              clippy

              # Linter
              reuse

              # LSPs
              rust-analyzer
              emmet-language-server
              nodePackages.vscode-langservers-extracted
            ];

            shellHook = ''echo "You have now entered the dev shell for Vel, exit at any time."'';
          };
        }
      );
    };
}
