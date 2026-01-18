{
  pkgs,
  treefmt-nix,
  imp-fmt-lib,
  ...
}:
let
  formatterEval = imp-fmt-lib.makeEval {
    inherit pkgs treefmt-nix;
    excludes = [
      "target/*"
      "**/target/*"
    ];
    rust = true;
  };
in
formatterEval.config.build.wrapper
