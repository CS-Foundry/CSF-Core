{
  description = "CSF-Core Master Node NixOS ISO";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  };

  outputs = { self, nixpkgs }: {
    nixosConfigurations.iso = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./modules/iso-configuration.nix
      ];
    };

    # Build command: nix build .#nixosConfigurations.iso.config.system.build.isoImage
    packages.x86_64-linux.default = 
      self.nixosConfigurations.iso.config.system.build.isoImage;
  };
}
