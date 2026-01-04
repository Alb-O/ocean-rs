# Rust packages
# Merged with other packages.d/ fragments
{
  __inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  __functor =
    _:
    {
      pkgs,
      rust-overlay,
      rootSrc,
      ...
    }:
    let
      cargoToml = builtins.fromTOML (builtins.readFile (rootSrc + "/Cargo.toml"));
      version = cargoToml.workspace.package.version;
      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile (rootSrc + "/rust-toolchain.toml");
      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };
    in
    {
      default = rustPlatform.buildRustPackage {
        pname = "@crate_name@";
        inherit version;
        src = rootSrc;
        cargoLock.lockFile = rootSrc + "/Cargo.lock";
        # Uncomment to build specific crate in workspace:
        # buildAndTestSubdir = "crates/YOUR_CRATE";
      };
    };
}
