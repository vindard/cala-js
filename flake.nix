{
  description = "@galoymoney/cala-ledger — napi-rs wrapper for cala-ledger";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};
    in {
      devShells.default = pkgs.mkShell {
        packages = with pkgs; [
          just

          process-compose
          postgresql_18

          nodejs_22
          yarn-berry

          rustc
          cargo
          rust-analyzer
          rustfmt
          clippy
        ];

        shellHook = ''
          export PG_HOST=127.0.0.1
          export PG_PORT=''${PG_PORT:-5433}
          export PG_USER=cala
          export PG_DB=cala
          export PG_CON="postgres://$PG_USER@$PG_HOST:$PG_PORT/$PG_DB?sslmode=disable"
          export PGDATA="$PWD/.dev/pg/data"
        '';
      };
    });
}
