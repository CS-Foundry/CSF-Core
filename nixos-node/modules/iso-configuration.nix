{ config, pkgs, lib, ... }:

{
  imports = [
    <nixpkgs/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix>
  ];

  # System configuration
  system.stateVersion = "24.11";

  # Networking
  networking = {
    hostName = "csf-docker-test";
    firewall = {
      enable = true;
      allowedTCPPorts = [
        8080  # Test nginx container
      ];
    };
  };

  # Enable Docker
  virtualisation.docker.enable = true;

  # System packages
  environment.systemPackages = with pkgs; [
    # Docker tools
    docker-compose
    docker

    # Utilities
    curl
    wget
    vim
    htop
  ];

  # Auto-login as root on boot (for ISO convenience)
  services.getty.autologinUser = "root";

  # Docker Compose service for nginx test
  systemd.services.docker-compose-test = {
    description = "Docker Compose Test Service (nginx)";
    after = [ "docker.service" ];
    requires = [ "docker.service" ];
    wantedBy = [ "multi-user.target" ];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      WorkingDirectory = "/etc/docker-test";
      ExecStart = "${pkgs.docker-compose}/bin/docker-compose up -d";
      ExecStop = "${pkgs.docker-compose}/bin/docker-compose down";
    };
  };

  # Activation script to setup Docker Compose
  system.activationScripts.docker-setup = {
    text = ''
      # Create docker-compose directory
      mkdir -p /etc/docker-test

      # Create docker-compose.yml
      cat > /etc/docker-test/docker-compose.yml <<EOF
version: '3.8'
services:
  nginx-test:
    image: nginx:alpine
    ports:
      - "8080:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./html:/usr/share/nginx/html:ro

volumes:
  nginx-logs:
EOF

      # Create nginx config
      cat > /etc/docker-test/nginx.conf <<EOF
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
      cat > /etc/docker-test/html/index.html <<EOF
<!DOCTYPE html>
<html>
<head>
    <title>CSF-Core Docker Test</title>
    <style>
        body { font-family: Arial, sans-serif; text-align: center; padding: 50px; }
        .container { max-width: 600px; margin: 0 auto; }
        h1 { color: #333; }
        .status { color: green; font-weight: bold; }
    </style>
</head>
<body>
    <div class="container">
        <h1>CSF-Core Docker Test</h1>
        <p class="status">Docker & Docker Compose funktionieren!</p>
        <p>Diese Seite wird von nginx in einem Docker Container serviert.</p>
        <p><a href="/health">Health Check</a></p>
    </div>
</body>
</html>
EOF

      # Create test script
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

  # Boot message with logo
  environment.etc."issue".text = ''

                                                                                
                                                                                
                                                                                
                        ..,,,,,,,,,,,,,,,,,,,,,,,;,,,,,,,,,,,,,,'..   .         
                      ..ckXXNNNNNNNNNNNNNNNNNNNNNNNNNNXXXXXXXXKx;.              
                    ..cONWMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMWOc..               
                  ..ckNWMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMW0l..    .  ..        
                ..ckNWMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMW0l..   .               
              ..ckXWMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMMW0l..                     
          .  .;kXWMMMMMMMMWKOkkkkkkkkkkkkkkkkkkkkkkkkxc..   .                   
             .oNWMMMMMMMMNx,..........................                          
             .oNMMMMMMMMMK:.                                                    
             .oNMMMMMMMMMK;         .........................                   
             .oNMMMMMMMMMK;       .'lddddddddddddddddddddddc'.  .               
             .oNMMMMMMMMMK;     .'o0NWWWWWWWWWWWWWWWWWWWWKd,.                   
             .oNMMMMMMMMMK;   .'l0NWMMMMMMMMMMMMMMMMMMWXx,.                     
             .oNMMMMMMMMMK; .'l0NWMMMMMMMMMMMMMMMMMMWXx;.                       
             .oNWMMMMMMMMK:'l0NWMMMMMMMMMMMMMMMMMMWXx;.                         
             .cKWMMMMMMMMXOOXWMMMMWWWWWWWWWWWWWWWXx;.                           
              .,dKWMMMMMMXo;lKMMMNOl:;;;;;;;;;;;;,.                             
                .,dKWMMMM0; 'kWMMMNOl'.                                         
                  .,dKWMM0; 'kWMMMMMN0o,...                                     
                    .,dKW0; 'kWMMMMMMMWKkc.                                     
                      .,ox, 'kWMMMMMMMMNXo.                                     
                        ... 'kWMMMMMMMMNXd.                                     
                            'kWMMMMMMMMNXd.                                     
                            'kWMMMMMMMMNXd.                                     
                            'kWMMMMMMMMNXo.                                     
                            'kWMMMMMMMMNXo.                                     
                            'kWMMMMMMMMNXo.                                     
                            'kWMMMMMMMNkl;.                                     
                            'kWMMMMMNk:...                                      
                            'kWMMMNk:.                                          
                            'kWWXx;.                                            
                        .   'kKx;.                                              
                            .;,.                                                
                            ..                                                  
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                
                                                                                

    ╔═══════════════════════════════════════════════════════════╗
    ║                                                           ║
    ║              CSF-Core Docker Test ISO                     ║
    ║                                                           ║
    ║  Einfache Docker & Docker Compose Testumgebung            ║
    ║                                                           ║
    ║  Services:                                                ║
    ║    - Docker:            systemctl status docker           ║
    ║    - Nginx Test:        http://localhost:8080             ║
    ║                                                           ║
    ║  Test commands:                                           ║
    ║    ./test-docker.sh    - Run comprehensive test           ║
    ║    docker ps -a        - List containers                  ║
    ║    docker-compose ps   - Compose status                   ║
    ║                                                           ║
    ╚═══════════════════════════════════════════════════════════╝

  '';
}