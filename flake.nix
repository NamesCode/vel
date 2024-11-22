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
      system = "aarch64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.aarch64-linux.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          # Rust tooling
          rustc
          rustfmt
          cargo
          clippy

          # LSPs
          rust-analyzer
          emmet-language-server
          nodePackages.vscode-langservers-extracted
        ];

        shellHook = ''echo "You have now entered the dev shell for Vel, exit at any time."'';
      };
    };
}
