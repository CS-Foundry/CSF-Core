{
  description = "CSF-Core Master Node NixOS Configuration";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  };

  outputs = { self, nixpkgs }: {
    # ISO-Konfiguration (für Installation)
    nixosConfigurations.iso = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./modules/iso-configuration.nix
      ];
    };

    # Server-Konfiguration (für Deployment)
    nixosConfigurations.csf-server = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux"; # oder "aarch64-linux" für ARM/Raspberry Pi
      modules = [
        ./modules/server-configuration.nix
      ];
    };

    # Build commands:
    # ISO: nix build .#nixosConfigurations.iso.config.system.build.isoImage
    packages.x86_64-linux = {
      iso = self.nixosConfigurations.iso.config.system.build.isoImage;
      default = self.nixosConfigurations.iso.config.system.build.isoImage;
    };
  };
}
