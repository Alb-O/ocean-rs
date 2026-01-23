/**
  Project-specific md-docs config

  Disabled rules:
  - MD013: Line length - not useful for docs with long URLs/code
  - MD033: Inline HTML - needed for badges, alignment, etc.
  - MD041: First line should be heading (from bundle default)
*/
{
  disabledRules = [ "MD013" "MD033" "MD041" ];
  exclude = [ "target" "result" "dist" "refs/" ];
}
