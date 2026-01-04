/**
  ast-grep rule helpers.

  Defines mkRule for creating ast-grep rules in Nix.
  Rules are converted to JSON then to YAML via yq.
*/
{ lib }:
{
  # Filter out empty lists for cleaner YAML output
  toJson = rule: builtins.toJSON (lib.filterAttrs (_: v: v != null && v != [ ]) rule);

  # Helper to define a rule with defaults
  mkRule =
    {
      id,
      language,
      message,
      rule,
      severity ? "warning",
      files ? [ ],
      ignores ? [ ],
    }:
    {
      inherit
        id
        language
        severity
        message
        files
        ignores
        rule
        ;
    };
}
