{ mkCommandRule, ... }:
mkCommandRule {
  id = "clippy";
  severity = "warning";
  message = "Clippy warning";
  run = "cargo clippy --all-targets --all-features --message-format=json -- -W missing_docs -W clippy::missing_docs_in_private_items";
}
