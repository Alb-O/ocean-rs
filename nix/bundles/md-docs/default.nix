/**
  Markdown linter and formatter using rumdl.

  A high-performance Markdown linter written in Rust.
  Supports GFM, MkDocs, MDX, and Quarto flavors.

  Disabled rules (sensible defaults):
  - MD013: Line length - not useful for docs with long URLs/code
  - MD033: Inline HTML - needed for badges, alignment, etc.
  - MD041: First line should be heading - not always applicable

  Project can add .rumdl.toml for additional config (excludes, etc.)
*/
{ pkgs, rootSrc, ... }:
let
  rumdl = pkgs.lib.getExe pkgs.rumdl;
  disableArgs = "MD013,MD033,MD041";
in
{
  __outputs.perSystem.formatter = {
    settings.formatter.rumdl = {
      command = rumdl;
      options = [
        "fmt"
        "--disable"
        disableArgs
      ];
      includes = [ "*.md" ];
    };
  };

  __outputs.perSystem.checks.markdown = pkgs.runCommand "rumdl-check" { } ''
    cd ${rootSrc}
    ${rumdl} check . --disable ${disableArgs} && touch $out
  '';
}
