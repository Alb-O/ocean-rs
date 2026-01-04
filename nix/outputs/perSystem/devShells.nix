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
      self',
      ...
    }:
    let
      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile (rootSrc + "/rust-toolchain.toml");
    in
    {
      default = pkgs.mkShell {
        # Compose with other devshells (e.g., lintfra) via devShells.d/
        inputsFrom = builtins.attrValues (builtins.removeAttrs self'.devShells [ "default" ]);

        packages = [
          rustToolchain
          pkgs.cargo-watch
          pkgs.cargo-insta
          pkgs.rust-analyzer
          self'.formatter

          # Linker dependencies (for fast builds)
          pkgs.mold
          pkgs.clang

          # Add project-specific dependencies here:
          # pkgs.pkg-config
          # pkgs.openssl
        ];

        shellHook = ''
          if [ -t 0 ]; then
            echo ""
            echo "Rust dev shell"
            echo "  Rust: $(rustc --version)"
            echo "  Cargo: $(cargo --version)"
          fi
        '';
      };
    };
}
