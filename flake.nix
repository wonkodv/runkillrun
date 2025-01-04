{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils?ref=c1dfcf08411b08f6b8615f7d8971a2bfa81d5e8a";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        buildDeps = with pkgs; [
          cargo
          rustc
        ];
        deps = with pkgs; [
          cargo-expand
          gdb
          clippy
          rustfmt
          rust-analyzer
          pre-commit
        ];
      in
      {
        devShells.default = pkgs.mkShell { buildInputs = deps ++ buildDeps; };
      }
      )
    ;
}
