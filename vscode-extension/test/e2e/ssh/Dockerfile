FROM ubuntu:24.04
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl openssh-server tmux gzip tar && rm -rf /var/lib/apt/lists/* && rm -f /etc/ssh/ssh_host_* \
 && useradd --create-home --shell /bin/bash agent && passwd -d agent \
 && mkdir -p /run/sshd /home/agent/work /home/agent/.ssh && chmod 700 /home/agent/.ssh && chown -R agent:agent /home/agent
CMD ["sh", "-ec", "ssh-keygen -A; exec /usr/sbin/sshd -D -e -o PasswordAuthentication=no -o KbdInteractiveAuthentication=no -o PermitRootLogin=no -o AllowUsers=agent"]
