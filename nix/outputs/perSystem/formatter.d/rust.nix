{ pkgs, ... }:
{
  programs.rustfmt.enable = true;
  settings.global.excludes = [
    "target/*"
    "**/target/*"
    "vendor/*"
  ];
}
