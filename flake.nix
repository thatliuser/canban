{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "nixpkgs";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in
    {
      packages."${system}" = rec {
        canban = pkgs.stdenv.mkDerivation {
          pname = "canban";
          version = "0.0.1";
          nativeBuildInputs = with pkgs; [
            openssl
            pkg-config
          ];
        };
        default = canban;
      };
    };
}
