{
  pkgs,
  treefmt-nix,
  imp-fmt,
  ...
}:
let
  formatterEval = imp-fmt.lib.makeEval {
    inherit pkgs treefmt-nix;
    excludes = [
      "target/*"
      "**/target/*"
    ];
    rust.enable = true;
  };
in
formatterEval.config.build.wrapper
