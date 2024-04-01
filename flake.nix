{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems = {
      url = "github:nix-systems/default";
      flake = false;
    };
  };
  outputs = { self, nixpkgs, rust-overlay, systems }:
    let
      inherit (nixpkgs) lib;
      eachSystem = lib.genAttrs (import systems);
      pkgsFor = eachSystem (system:
        import nixpkgs {
          localSystem = system;
          overlays = [ rust-overlay.overlays.default ];
        });
    in
    {
      devShells = eachSystem (system:
        let
          pkgs = pkgsFor.${system};
          rust-version = "latest";
          rust-toolchain = pkgs.rust-bin.stable.${rust-version}.default.override
            {
              extensions = [
                "rust-src"
                "rust-analyzer"
              ];
            };
        in
        {
          default = with pkgs; mkShell {
            buildInputs = [
              openssl
              pkg-config
              rust-toolchain
            ];
          };
        });

      formatter = eachSystem (system: pkgsFor.${system}.nixpkgs-fmt);
    };
}
