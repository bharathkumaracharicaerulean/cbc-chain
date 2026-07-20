#!/bin/bash
# docker_setup.sh - Install and configure Docker and Docker Compose for the CBC-Chain project.
# Supports Debian/Ubuntu systems and macOS.
# Run this to set up the containerized development environment.

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}        CBC-Chain Docker Setup          ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Identify OS / Distribution
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS_NAME=$ID
    OS_LIKE=$ID_LIKE
else
    OS_NAME=$(uname -s)
    OS_LIKE=""
fi

# Detect platform and route installation
if [[ "$OS_NAME" == "ubuntu" || "$OS_NAME" == "debian" || "$OS_LIKE" == *"ubuntu"* || "$OS_LIKE" == *"debian"* ]]; then
    echo -e "${YELLOW}Debian/Ubuntu-based system detected ($OS_NAME). Setting up Docker...${NC}"
    
    # 1. Remove conflicting packages
    echo -e "${YELLOW}[1/5] Removing conflicting older Docker packages...${NC}"
    for pkg in docker.io docker-doc docker-compose docker-compose-v2 podman-docker containerd runc; do
        sudo apt-get remove -y $pkg 2>/dev/null || true
    done
    echo -e "${GREEN}✓ Conflict cleanup completed.${NC}"

    # 2. Add Docker's official GPG key
    echo -e "${YELLOW}[2/5] Setting up Docker GPG key...${NC}"
    sudo rm -f /etc/apt/sources.list.d/docker.list
    sudo apt-get update -qq
    sudo apt-get install -y ca-certificates curl
    sudo install -m 0755 -d /etc/apt/keyrings
    
    # Determine OS name for the download URL (map derivative OS to parent if needed)
    DOCKER_OS="$OS_NAME"
    if [[ "$DOCKER_OS" != "ubuntu" && "$DOCKER_OS" != "debian" ]]; then
        if [[ "$OS_LIKE" == *"ubuntu"* ]]; then
            DOCKER_OS="ubuntu"
        elif [[ "$OS_LIKE" == *"debian"* ]]; then
            DOCKER_OS="debian"
        fi
    fi

    sudo curl -fsSL "https://download.docker.com/linux/$DOCKER_OS/gpg" -o /etc/apt/keyrings/docker.asc
    sudo chmod a+r /etc/apt/keyrings/docker.asc
    echo -e "${GREEN}✓ Docker GPG key added.${NC}"

    # 3. Add the repository to Apt sources
    echo -e "${YELLOW}[3/5] Setting up Docker Apt repository...${NC}"
    # Use VERSION_CODENAME or fall back to Ubuntu/Debian equivalents
    CODENAME="$VERSION_CODENAME"
    if [ -z "$CODENAME" ]; then
        CODENAME=$(lsb_release -cs 2>/dev/null || echo "stable")
    fi

    echo \
      "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/$DOCKER_OS \
      $CODENAME stable" | \
      sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
    
    sudo apt-get update -qq
    echo -e "${GREEN}✓ Docker Apt repository configured.${NC}"

    # 4. Install Docker packages
    echo -e "${YELLOW}[4/5] Installing Docker Engine and Docker Compose...${NC}"
    sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
    echo -e "${GREEN}✓ Docker packages installed.${NC}"

    # 5. Configure user permissions & Docker daemon
    echo -e "${YELLOW}[5/5] Configuring post-installation steps...${NC}"
    
    # Start and enable docker service
    if command -v systemctl &>/dev/null; then
        echo "Starting and enabling Docker service..."
        sudo systemctl enable --now docker
    fi

    # Add current user to docker group if not already present
    if ! groups "$USER" | grep -q "\bdocker\b"; then
        echo -e "${YELLOW}Adding user $USER to the docker group...${NC}"
        sudo usermod -aG docker "$USER"
        echo -e "${GREEN}✓ User added to the docker group.${NC}"
        NEED_GROUP_RELOAD=true
    else
        echo -e "${GREEN}✓ User is already in the docker group.${NC}"
        NEED_GROUP_RELOAD=false
    fi

elif [[ "$OS_NAME" == "Darwin" ]]; then
    echo -e "${YELLOW}macOS detected. Setting up Docker via Homebrew...${NC}"
    
    if ! command -v brew &>/dev/null; then
        echo -e "${RED}Homebrew is not installed. Please install Homebrew or Docker Desktop manually.${NC}"
        exit 1
    fi

    echo -e "${YELLOW}Installing Docker CLI and Docker Compose...${NC}"
    brew install docker docker-compose
    echo -e "${GREEN}✓ Homebrew packages installed.${NC}"
    
    echo -e "${BLUE}Note: On macOS, Docker requires a backend/hypervisor. We recommend installing Docker Desktop:${NC}"
    echo "  brew install --cask docker"
    echo "Alternatively, you can use Colima as a lightweight VM backend:"
    echo "  brew install colima && colima start"
    NEED_GROUP_RELOAD=false
else
    echo -e "${RED}Unsupported OS: $OS_NAME ($OS_LIKE)${NC}"
    echo "Please install Docker manually using the instructions at:"
    echo "  https://docs.docker.com/engine/install/"
    exit 1
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}      Docker Setup completed!           ${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "Verified versions:"
docker --version || echo "Docker CLI not fully configured yet"
docker compose version || echo "Docker Compose plugin not fully configured yet"
echo ""

if [ "$NEED_GROUP_RELOAD" = true ]; then
    echo -e "${YELLOW}IMPORTANT:${NC} To run Docker commands without 'sudo', please log out and back in,"
    echo -e "or run the following command in your terminal to apply the group changes immediately:"
    echo -e "  ${GREEN}newgrp docker${NC}"
    echo ""
fi

echo -e "Now you can run the local monitoring node or validation network:"
echo -e "  ${BLUE}docker compose up -d${NC}"
echo ""
