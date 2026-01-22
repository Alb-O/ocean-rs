/**
  imp-lint bundle: ast-grep based linting with custom rules.

  Provides:
  - packages.imp-lint: Nushell module with injected rules
  - packages.imp-lint-rules: Generated YAML rules
  - packages.imp-lint-rules-sync: Utility to sync rules locally
  - devShells.imp-lint: Shell with ast-grep and nushell
  - checks.build: Package build check
*/
{
  pkgs,
  lib,
  self,
  self',
  ...
}:
let
  astGrep = import "${self}/nix/lib/ast-grep-rule.nix" { inherit lib; };
  customRule = import "${self}/nix/lib/custom-rule.nix" { inherit lib; };

  # ast-grep rules from lint/rules/
  rulesDir = "${self}/lint/rules";
  ruleFiles = builtins.filter (f: lib.hasSuffix ".nix" f) (
    builtins.attrNames (builtins.readDir rulesDir)
  );
  rules = map (f: {
    name = lib.removeSuffix ".nix" f;
    rule = import (rulesDir + "/${f}") { inherit (astGrep) mkRule; };
  }) ruleFiles;

  # Generate YAML files
  generatedRules = pkgs.runCommand "ast-grep-rules" { buildInputs = [ pkgs.yq-go ]; } ''
    mkdir -p $out
    ${lib.concatMapStringsSep "\n" (
      r: ''echo '${astGrep.toJson r.rule}' | yq -P > $out/${r.name}.yml''
    ) rules}
  '';

  # custom rules from lint/custom/
  customRulesDir = "${self}/lint/custom";
  customRuleFiles = builtins.filter (f: lib.hasSuffix ".nix" f) (
    builtins.attrNames (builtins.readDir customRulesDir)
  );
  customRules = map (f: import (customRulesDir + "/${f}") customRule) customRuleFiles;
  customRulesJson = builtins.toJSON customRules;

  # Nushell module package
  moduleScript = "${self}/nix/scripts/imp-lint.nu";
  impLintModule = pkgs.runCommand "imp-lint" { } ''
    mkdir -p $out/lib
    substitute ${moduleScript} $out/lib/imp-lint \
      --replace-warn '@impLintRules@' '${generatedRules}' \
      --replace-warn '@impLintRulesInjected@' 'true' \
      --replace-warn "@impLintCustomRules@" '${customRulesJson}' \
      --replace-warn '@impLintCustomRulesInjected@' 'true'
  '';
in
{
  __outputs.perSystem.packages = {
    imp-lint = impLintModule;
    default = impLintModule;
    imp-lint-rules = generatedRules;
    imp-lint-rules-sync = pkgs.writeShellScriptBin "imp-lint-rules-sync" ''
      set -e
      dest="''${1:-lint/ast-rules}"
      mkdir -p "$dest"
      rm -f "$dest"/*.yml
      cp ${generatedRules}/*.yml "$dest/"
      echo "Synced ${toString (builtins.length rules)} rules to $dest"
    '';
  };

  __outputs.perSystem.devShells.imp-lint = pkgs.mkShell {
    packages = [
      pkgs.ast-grep
      pkgs.nushell
      impLintModule
    ];

    shellHook = ''
      if [ -t 0 ] && [ -d .git ]; then
        if [ -f ./nix/scripts/pre-commit.nu ]; then
          cat > .git/hooks/pre-commit << 'EOF'
#!/usr/bin/env bash
exec nu ./nix/scripts/pre-commit.nu "$@"
EOF
          chmod +x .git/hooks/pre-commit
        elif [ -x ./nix/scripts/pre-commit ]; then
          cp ./nix/scripts/pre-commit .git/hooks/pre-commit
          chmod +x .git/hooks/pre-commit
        fi
      fi
      echo "imp-lint: ast-grep + clippy + custom rules"
    '';
  };

  __outputs.perSystem.checks.build = impLintModule;
}
