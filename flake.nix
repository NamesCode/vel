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
          rustc
          rustfmt
          cargo
          clippy
          rust-analyzer
        ];

        shellHook = ''echo "You have now entered the dev shell for Velox, exit at any time."'';
      };
    };
}
