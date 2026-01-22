# Flake outputs entry point
# This file is referenced by the auto-generated flake.nix
inputs:
let
  inherit (inputs)
    nixpkgs
    flake-parts
    imp
    ;
in
flake-parts.lib.mkFlake { inherit inputs; } {
  imports = [
    imp.flakeModules.default
  ];

  systems = import inputs.systems;

  # Apply rust-overlay to pkgs for all perSystem modules
  perSystem =
    { system, ... }:
    {
      _module.args.pkgs = import nixpkgs {
        inherit system;
        overlays = [ inputs.rust-overlay.overlays.default ];
      };
    };

  # imp configuration
  imp = {
    src = ../outputs;
    bundles.src = ../bundles;

    # Extra args available in all output files
    args = {
      inherit nixpkgs;
      rootSrc = ../..;
      rust-overlay = inputs.rust-overlay;
    };

    # Disable exports (not used in this project)
    exports.enable = false;

    # Auto-generate default devShell composing all bundle devShells
    impShell.enable = true;

    # Auto-generate flake.nix from __inputs declarations
    flakeFile = {
      enable = true;
      coreInputs = import ./inputs.nix;
      description = "@project_name@";
      outputsFile = "./nix/flake";
    };
  };
}
