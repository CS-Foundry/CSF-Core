{ config, pkgs, lib, ... }:

{
  # System configuration - WICHTIG: Muss mit der ursprünglichen Installation übereinstimmen!
  system.stateVersion = "25.11";

  # Boot configuration
  boot = {
    loader.grub = {
      enable = true;
      device = "/dev/sda";
      useOSProber = true;
    };
    
    # Hardware-spezifische Einstellungen (von hardware-configuration.nix)
    initrd.availableKernelModules = [ "ata_piix" "uhci_hcd" "virtio_pci" "virtio_scsi" "sd_mod" "sr_mod" ];
    initrd.kernelModules = [ ];
    kernelModules = [ ];
    extraModulePackages = [ ];
  };

  # File Systems (von hardware-configuration.nix)
  fileSystems."/" = {
    device = "/dev/disk/by-uuid/e4b27226-e75f-4cef-9dec-fc0c6f2185ac";
    fsType = "ext4";
  };

  swapDevices = [ ];

  # Platform
  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";

  # Networking
  networking = {
    hostName = "nixos"; # Match existing hostname
    
    # NetworkManager aktivieren (wie auf dem Zielsystem)
    networkmanager.enable = true;
    
    firewall = {
      enable = true;
      allowedTCPPorts = [
        22    # SSH
        80    # HTTP
        443   # HTTPS
        8080  # Docker nginx test
      ];
    };
  };

  # Time zone
  time.timeZone = "Europe/Berlin";

  # Locale settings
  i18n.defaultLocale = "de_DE.UTF-8";
  i18n.extraLocaleSettings = {
    LC_ADDRESS = "de_DE.UTF-8";
    LC_IDENTIFICATION = "de_DE.UTF-8";
    LC_MEASUREMENT = "de_DE.UTF-8";
    LC_MONETARY = "de_DE.UTF-8";
    LC_NAME = "de_DE.UTF-8";
    LC_NUMERIC = "de_DE.UTF-8";
    LC_PAPER = "de_DE.UTF-8";
    LC_TELEPHONE = "de_DE.UTF-8";
    LC_TIME = "de_DE.UTF-8";
  };

  # Console keymap
  console.keyMap = "de";

  # X11 keymap
  services.xserver.xkb = {
    layout = "de";
    variant = "";
  };

  # SSH Server für Remote-Zugriff
  services.openssh = {
    enable = true;
    settings = {
      PermitRootLogin = "yes"; # Match existing config
      PasswordAuthentication = true; # Match existing config
    };
  };

  # Bestehenden User rootcsf übernehmen
  users.users.rootcsf = {
    isNormalUser = true;
    description = "rootcsf";
    extraGroups = [ "networkmanager" "wheel" "docker" ];
    packages = with pkgs; [];
  };

  # Sudo ohne Passwort für wheel-Gruppe (für automatisiertes Deployment)
  security.sudo.wheelNeedsPassword = false;

  # GnuPG Agent (wie auf dem Zielsystem)
  programs.mtr.enable = true;
  programs.gnupg.agent = {
    enable = true;
    enableSSHSupport = true;
  };

  # Docker aktivieren
  virtualisation.docker = {
    enable = true;
    enableOnBoot = true;
  };

  # System packages
  environment.systemPackages = with pkgs; [
    # Docker tools
    docker-compose

    # System utilities
    curl
    wget
    vim
    htop
    git
    tmux
    
    # Debugging tools
    lsof
    netcat
    tcpdump
  ];

  # Docker Compose service for nginx test
  systemd.services.docker-compose-test = {
    description = "Docker Compose Test Service (nginx)";
    after = [ "docker.service" "network-online.target" ];
    requires = [ "docker.service" "network-online.target" ];
    wantedBy = [ "multi-user.target" ];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      WorkingDirectory = "/etc/docker-test";
      ExecStart = "${pkgs.docker-compose}/bin/docker-compose up -d";
      ExecStop = "${pkgs.docker-compose}/bin/docker-compose down";
      Restart = "on-failure";
    };
  };

  # Activation script to setup Docker Compose
  system.activationScripts.docker-setup = {
    text = ''
      # Create docker-compose directory
      mkdir -p /etc/docker-test

      # Create docker-compose.yml
      cat > /etc/docker-test/docker-compose.yml <<'EOF'
version: '3.8'
services:
  nginx-test:
    image: nginx:alpine
    ports:
      - "8080:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./html:/usr/share/nginx/html:ro
    restart: unless-stopped

volumes:
  nginx-logs:
EOF

      # Create nginx config
      cat > /etc/docker-test/nginx.conf <<'EOF'
events {
    worker_connections 1024;
}

http {
    server {
        listen 80;
        server_name localhost;

        location / {
            root /usr/share/nginx/html;
            index index.html;
        }

        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }
    }
}
EOF

      # Create HTML content
      mkdir -p /etc/docker-test/html
      cat > /etc/docker-test/html/index.html <<'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>CSF-Core Server</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; padding: 50px; }
        .container { max-width: 600px; margin: 0 auto; }
        h1 { color: #333; }
        .status { color: green; font-weight: bold; }
    </style>
</head>
<body>
    <div class="container">
        <h1>CSF-Core Server läuft!</h1>
        <p class="status">Docker & Docker Compose funktionieren!</p>
        <p>Diese Seite wird von nginx in einem Docker Container serviert.</p>
        <p><a href="/health">Health Check</a></p>
    </div>
</body>
</html>
EOF

      # Create test script
      cat > /root/test-docker.sh <<'EOF'
#!/bin/bash
echo "=== CSF-Core Docker Test ==="
echo "Hostname: $(hostname)"
echo "Date: $(date)"
echo ""
echo "Docker version:"
docker --version
echo ""
echo "Docker Compose version:"
docker-compose --version
echo ""
echo "Docker images:"
docker images
echo ""
echo "Running containers:"
docker ps
echo ""
echo "Docker Compose status:"
cd /etc/docker-test && docker-compose ps
echo ""
echo "=== Network Test ==="
echo "Testing nginx container:"
curl -s http://localhost:8080 | grep -o '<title>.*</title>' || echo "Port 8080 not responding"
echo ""
echo "Health check:"
curl -s http://localhost:8080/health || echo "Health check failed"
echo ""
echo "=== Test Complete ==="
EOF
      chmod +x /root/test-docker.sh
    '';
    deps = [];
  };

  # Automatic updates (optional, aber empfohlen)
  system.autoUpgrade = {
    enable = false; # Auf true setzen für automatische Updates
    dates = "04:00";
    allowReboot = false;
  };

  # Nix settings
  nix = {
    settings = {
      experimental-features = [ "nix-command" "flakes" ];
      auto-optimise-store = true;
    };
    gc = {
      automatic = true;
      dates = "weekly";
      options = "--delete-older-than 30d";
    };
  };
}
