{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    systems = {
      url = "github:nix-systems/default";
      flake = false;
    };
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = { self, nixpkgs, rust-overlay, systems, crane }:
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
      apps = eachSystem (system:
        let
          pkgs = pkgsFor.${system};
          craneLib = crane.mkLib pkgs;
          kuvibot = craneLib.buildPackage {
            src = craneLib.cleanCargoSource ./.;
            buildInputs = with pkgs;[
              pkg-config
              openssl
            ];
            BOT_KS = ./src/bot.ks;
          };
        in
        {
          default = {
            type = "app";
            program = "${kuvibot}/bin/kuvibot";
          };
        });
      devShells = eachSystem (system:
        let
          pkgs = pkgsFor.${system};
          rust-stable = pkgs.rust-bin.stable.latest.minimal.override {
            extensions = [ "rust-src" "rust-docs" "clippy" ];
          };
        in
        {
          default = with pkgs;
            mkShell {
              strictDeps = true;
              packages = [
                # Derivations in `rust-stable` take precedence over nightly.
                (lib.hiPrio rust-stable)

                # Use rustfmt, and other tools that require nightly features.
                (pkgs.rust-bin.selectLatestNightlyWith
                  (toolchain:
                    toolchain.minimal.override {
                      extensions = [ "rustfmt" "rust-analyzer" ];
                    }))

                # Native transitive dependencies for Cargo
                pkg-config
                openssl
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
