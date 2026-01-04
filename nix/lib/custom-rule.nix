/**
  Custom lint rule helpers (clippy, file-metric, etc.)
*/
{ lib }:
{
  toJson = rule: builtins.toJSON (lib.filterAttrs (_: v: v != null && v != [ ]) rule);

  mkCommandRule =
    {
      id,
      run,
      message,
      severity ? "warning",
    }:
    {
      inherit
        id
        severity
        message
        run
        ;
      type = "command";
    };

  mkFileMetricRule =
    {
      id,
      check,
      max,
      message,
      files,
      severity ? "warning",
      ignores ? [ ],
    }:
    {
      inherit
        id
        severity
        message
        check
        max
        files
        ignores
        ;
      type = "file-metric";
    };
}
