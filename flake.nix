{
  description = "*Run* an executable Program. When the exe file changes *kill* the old process. *Run* the new exe again.";

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
        deps = with pkgs; [
          cargo
          rustc
          cargo-expand
          gdb
          clippy
          rustfmt
          rust-analyzer
          pre-commit
        ];
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "rkr";
          version = "0.1.0"; # keep in synch with Cargo.toml version
          src = ./.;

          cargoHash = "sha256-yS9jl2PX0uXWtYecmKy62DY7q/xhdYb/DkMrpkbybUk=";

          meta = with pkgs.lib; {
            description = "A simple Rust CLI application";
            homepage = "https://github.com/wonkodv/runkillrun";
          };
        };

        devShells.default = pkgs.mkShell { buildInputs = deps; };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/rkr";
        };
      }
    );
}
