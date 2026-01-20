{ config, pkgs, lib, ... }:

{
  imports = [
    <nixpkgs/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix>
  ];

  # System configuration
  system.stateVersion = "24.11";

  # Networking
  networking = {
    hostName = "csf-master-node";
    firewall = {
      enable = true;
      allowedTCPPorts = [
        8000  # CSF-Core Backend API
        3000  # CSF-Core Frontend
        8443  # P2P Agent communication
        8080  # Test Docker container
      ];
    };
  };

  # Enable Docker
  virtualisation.docker.enable = true;

  # System packages
  environment.systemPackages = with pkgs; [
    # Build tools
    rustc
    cargo
    gcc
    pkg-config
    openssl

    # Node.js for frontend
    nodejs_20

    # Utilities
    curl
    wget
    git
    vim
    htop
    tmux

    # Docker tools
    docker-compose
    docker

    # Additional tools
    jq
    tree
  ];

  # Users
  users.users.csf-core = {
    isSystemUser = true;
    group = "csf-core";
    extraGroups = [ "docker" ];
    home = "/opt/csf-core";
    createHome = true;
  };

  users.groups.csf-core = {};

  # Auto-login as root on boot (for ISO convenience)
  services.getty.autologinUser = "root";

  # CSF-Core systemd service
  systemd.services.csf-core = {
    description = "CSF Core Backend and Frontend Service";
    documentation = [ "https://github.com/CS-Foundry/CSF-Core" ];
    after = [ "network.target" ];
    wantedBy = [ "multi-user.target" ];

    serviceConfig = {
      Type = "simple";
      User = "csf-core";
      Group = "csf-core";
      WorkingDirectory = "/opt/csf-core";

      # Environment variables
      Environment = [
        "DATABASE_URL=sqlite:/opt/csf-core/finance.db"
        "JWT_SECRET=change-this-in-production"
        "RUST_LOG=info"
        "NODE_ENV=production"
        "PORT=3000"
        "FRONTEND_URL=http://localhost:3000"
        "ORIGIN=http://localhost:8000"
      ];

      # Security settings
      NoNewPrivileges = true;
      PrivateTmp = true;
      ProtectSystem = "strict";
      ProtectHome = true;
      ReadWritePaths = [
        "/opt/csf-core"
        "/var/lib/csf-core"
        "/var/log/csf-core"
      ];
      SupplementaryGroups = [ "docker" ];

      # Startup
      ExecStart = "/opt/csf-core/start.sh";

      # Process management
      Restart = "on-failure";
      RestartSec = 10;
      KillMode = "mixed";
      KillSignal = "SIGTERM";
      TimeoutStartSec = 60;
      TimeoutStopSec = 30;

      # Resource limits
      LimitNOFILE = 65536;
      LimitNPROC = 4096;
    };
  };

  # Test Docker container service
  systemd.services.test-docker-container = {
    description = "Test Docker Container (Nginx Hello World)";
    after = [ "docker.service" ];
    requires = [ "docker.service" ];
    wantedBy = [ "multi-user.target" ];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      ExecStart = "${pkgs.docker}/bin/docker run -d --name test-nginx -p 8080:80 nginx:alpine";
      ExecStop = "${pkgs.docker}/bin/docker stop test-nginx && ${pkgs.docker}/bin/docker rm test-nginx";
    };
  };

  # Activation script to setup CSF-Core and Docker on first boot
  system.activationScripts.setup = {
    text = ''
      # Create necessary directories
      mkdir -p /opt/csf-core
      mkdir -p /var/lib/csf-core
      mkdir -p /var/log/csf-core

      # Set ownership
      chown -R csf-core:csf-core /opt/csf-core
      chown -R csf-core:csf-core /var/lib/csf-core
      chown -R csf-core:csf-core /var/log/csf-core

      # Create config.env if not exists
      if [ ! -f /opt/csf-core/config.env ]; then
        cat > /opt/csf-core/config.env <<EOF
DATABASE_URL=sqlite:/opt/csf-core/finance.db
JWT_SECRET=change-this-in-production
RUST_LOG=info
NODE_ENV=production
PORT=3000
FRONTEND_URL=http://localhost:3000
ORIGIN=http://localhost:8000
EOF
        chown csf-core:csf-core /opt/csf-core/config.env
      fi

      # Create a simple test script
      cat > /root/test-docker.sh <<EOF
#!/bin/bash
echo "=== Docker Test Script ==="
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
docker ps -a
echo ""
echo "Test container logs:"
docker logs test-nginx 2>/dev/null || echo "No test container logs available"
echo ""
echo "=== CSF-Core Status ==="
systemctl status csf-core --no-pager -l || echo "CSF-Core service not running"
echo ""
echo "=== Network Test ==="
echo "Testing localhost ports:"
curl -s http://localhost:8080 | head -5 || echo "Port 8080 not responding"
echo ""
curl -s http://localhost:3000 | head -5 || echo "Port 3000 not responding"
echo ""
echo "=== Test Complete ==="
EOF
      chmod +x /root/test-docker.sh
    '';
    deps = [];
  };

  # Boot message
  environment.etc."issue".text = ''

    ╔═══════════════════════════════════════════════════════════╗
    ║                                                           ║
    ║              CSF-Core Master Node ISO                     ║
    ║                                                           ║
    ║  This system is configured to run CSF-Core automatically  ║
    ║  with Docker and a test container.                        ║
    ║                                                           ║
    ║  Services:                                                ║
    ║    - CSF-Core Backend:  http://localhost:8000             ║
    ║    - CSF-Core Frontend: http://localhost:3000             ║
    ║    - Test Container:    http://localhost:8080             ║
    ║    - Docker:            systemctl status docker           ║
    ║                                                           ║
    ║  Test commands:                                           ║
    ║    ./test-docker.sh    - Run comprehensive test           ║
    ║    docker ps -a        - List containers                  ║
    ║    docker logs test-nginx - View test container logs      ║
    ║                                                           ║
    ╚═══════════════════════════════════════════════════════════╝

  '';
}