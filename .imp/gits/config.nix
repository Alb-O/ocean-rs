{
  injections = [
    {
      name = "imp-lint";
      remote = "https://github.com/imp-nix/imp.lint.git";
      use = [
        "lint/rules"
        "lint/custom"
        "nix/scripts"
        "nix/lib"
        "nix/outputs/perSystem/packages.d/10-lint.nix"
        "nix/outputs/perSystem/packages.d/20-lint-rules.nix"
        "nix/outputs/perSystem/devShells.d/10-imp-lint.nix"
        "nix/outputs/perSystem/formatter.nix"
      ];
    }
    {
      name = "rust-boilerplate";
      remote = "https://github.com/Alb-O/rust-boilerplate.git";
      use = [
        "rust-toolchain.toml"
        "rustfmt.toml"
        "clippy.toml"
      ];
      boilerplate.dir = "boilerplate";
    }
  ];
}
