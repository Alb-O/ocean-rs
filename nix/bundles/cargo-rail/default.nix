/**
  cargo-rail: Graph-aware monorepo orchestration for Rust workspaces.

  Provides the cargo-rail package and adds it to the devShell.
*/
{ pkgs, ... }:
let
  cargo-rail = pkgs.rustPlatform.buildRustPackage rec {
    pname = "cargo-rail";
    version = "0.8.1";

    src = pkgs.fetchCrate {
      inherit pname version;
      hash = "sha256-O92YgxudBCE5A75EEBiZcE1KXSJE77q2HH6sDlvBk60=";
    };

    cargoHash = "sha256-mErr9bF4R4H7ByU5eFFCq8CHIAoBSUjM4IxVtIfyRLI=";

    nativeBuildInputs = [ pkgs.pkg-config ];

    buildInputs = [
      pkgs.curl
      pkgs.openssl
    ];

    doCheck = false;

    meta = {
      description = "Graph-aware monorepo orchestration for Rust workspaces";
      mainProgram = "cargo-rail";
      homepage = "https://github.com/loadingalias/cargo-rail";
      license = pkgs.lib.licenses.mit;
      platforms = pkgs.lib.platforms.unix;
    };
  };
in
{
  __outputs.perSystem.packages.cargo-rail = cargo-rail;

  __outputs.perSystem.devShells.cargo-rail = pkgs.mkShell {
    packages = [ cargo-rail ];
  };
}
