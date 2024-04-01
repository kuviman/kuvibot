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
            strictDeps = true;
            packages = [
              openssl
              pkg-config
              rust-toolchain
            ];
            OPENSSL_LIB_DIR = "${openssl.out}/lib";
            OPENSSL_ROOT_DIR = "${openssl.out}";
            OPENSSL_INCLUDE_DIR = "${openssl.dev}/include";
            # RUST_BACKTRACE = 1;
          };
        });

      formatter = eachSystem (system: pkgsFor.${system}.nixpkgs-fmt);
    };
}
