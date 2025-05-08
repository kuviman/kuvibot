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
          kuvibot = channel-name:
            let
              common-args = {
                src = craneLib.cleanCargoSource ./.;
                buildInputs = with pkgs;[
                  pkg-config
                  openssl
                ];
              };
            in
            craneLib.buildPackage (common-args // {
              cargoArtifacts = craneLib.buildDepsOnly common-args;
              BOT_KS = ./src-ks;
              CONFIG = ./config/${channel-name}.toml;
            });
          kuvibot-app = channel-name: {
            type = "app";
            program = "${kuvibot channel-name}/bin/kuvibot";
          };
        in
        {
          kuviman = kuvibot-app "kuviman";
          kuviboy = kuvibot-app "kuviboy";
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
