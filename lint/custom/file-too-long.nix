{ mkFileMetricRule, ... }:
mkFileMetricRule {
  id = "file-too-long";
  severity = "warning";
  message = "File exceeds 450 lines; split into smaller modules";
  check = "line-count";
  max = 450;
  files = [ "**/*.rs" ];
  ignores = [
    "**/target/**"
    "**/benches/**"
  ];
}
