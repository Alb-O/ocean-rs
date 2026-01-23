/**
  Markdown linter and formatter using rumdl.

  A high-performance Markdown linter written in Rust.
  Supports GFM, MkDocs, MDX, and Quarto flavors.
*/
{ pkgs, ... }:
{
  __outputs.perSystem.formatter = {
    settings.formatter.rumdl = {
      command = pkgs.lib.getExe pkgs.rumdl;
      options = [ "fmt" ];
      includes = [ "*.md" ];
    };
  };
}
