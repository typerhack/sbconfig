# lab/Dockerfile
# Ubuntu 22.04 lab environment for sbconfig development
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies (including build tools for Rust)
RUN apt-get update && apt-get install -y \
    openssh-server \
    curl \
    wget \
    sudo \
    sqlite3 \
    vim \
    nano \
    net-tools \
    iproute2 \
    ca-certificates \
    gnupg \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Configure bashrc for Ghostty terminal support
RUN echo 'export TERM=xterm-256color' >> /root/.bashrc

# Configure SSH - use config.d to ensure our settings take precedence
RUN mkdir /var/run/sshd && \
    echo 'root:labpass' | chpasswd && \
    echo "PermitRootLogin yes" > /etc/ssh/sshd_config.d/10-lab.conf && \
    echo "PasswordAuthentication yes" >> /etc/ssh/sshd_config.d/10-lab.conf

# Setup SSH key for passwordless access (key will be copied in via volume or script)
RUN mkdir -p /root/.ssh && chmod 700 /root/.ssh

# Install sing-box from official repository
RUN curl -fsSL https://sing-box.app/gpg.key -o /etc/apt/keyrings/sagernet.asc && \
    chmod a+r /etc/apt/keyrings/sagernet.asc && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/sagernet.asc] https://deb.sagernet.org/ * *" | \
    tee /etc/apt/sources.list.d/sagernet.list > /dev/null && \
    apt-get update && \
    apt-get install -y sing-box && \
    rm -rf /var/lib/apt/lists/*

# Create sbconfig directories
RUN mkdir -p /var/lib/sbconfig /var/log/sbconfig /usr/local/bin

# Create a basic sing-box config so the service can start
RUN mkdir -p /etc/sing-box && \
    echo '{"log":{"level":"info"}}' > /etc/sing-box/config.json

# Expose ports
# 22 = SSH (internal, mapped to 2222 on host)
# 7344 = Custom SSH port for sing-box proxy users (example)
# 10808 = sing-box proxy port
EXPOSE 22 7344 10808

# Start SSH
CMD ["/usr/sbin/sshd", "-D"]
