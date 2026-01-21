{
  __inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  __functor =
    _:
    { self', ... }:
    {
      # Package build implicitly runs tests via doCheck
      build = self'.packages.default;
    };
}
