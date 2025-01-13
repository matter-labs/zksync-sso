{
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

  outputs = {
    flake-utils,
    nixpkgs,
    self,
    ...
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
      };
    in
      with pkgs; let
        # Common environment variables
        commonEnv = {
          NUXT_TELEMETRY_DISABLED = 1;
        };

        # Environment-specific configurations
        envConfigs = {
          testnet = {
            chainId = 240;
            nftQuestAddress = "0x1DB051f0853c01EF57AE4Ef812282379778370C3";
          };
          mainnet = {
            chainId = 388;
            nftQuestAddress = "0xMainnetAddressHere";
          };
        };

        # Create environment variables for a specific environment
        mkEnvVars = env: commonEnv // {
          NUXT_PUBLIC_DEFAULT_CHAIN_ID = envConfigs.${env}.chainId;
          NUXT_PUBLIC_CRONOS_ZKEVM_NFT_QUEST_ADDRESS = envConfigs.${env}.nftQuestAddress;
        };

        # Create a derivation for the application
        mkApp = env:
          stdenv.mkDerivation (finalAttrs: {
            pname = "zksync-sso-auth-server";
            version = "1.0.0";
            
            src = ./.;

            nativeBuildInputs = [
              faketty
              nodejs
              pnpm.configHook
            ];

            env = mkEnvVars env;

            buildPhase = ''
              runHook preBuild
              faketty pnpm nx run auth-server:build
              runHook postBuild
            '';

            installPhase = ''
              runHook preInstall
              mkdir -p $out/dist
              cp -r ./packages/auth-server/dist/* $out/dist/
              runHook postInstall
            '';

            prePnpmInstall = ''
              pnpm install --force --frozen-lockfile
            '';

            pnpmDeps = pnpm.fetchDeps {
              inherit (finalAttrs) pname version src prePnpmInstall;
              hash = "sha256-AzQaJODjaDre1YCQShA71cuRcc2CCt5MNLjB80zmlQs=";
            };
          });
      in {
        packages = {
          # Build packages for each environment
          testnet = mkApp "testnet";
          mainnet = mkApp "mainnet";
        };
      });
}
