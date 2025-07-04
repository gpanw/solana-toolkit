# Use the official Rust image as a base image
FROM rust:1.85.0
#1.81.0

# Install Solana CLI
# version 1.18.26 is installed because it has to be in sync with geyser plugin
# RUN sh -c "$(curl -sSfL https://release.anza.xyz/v1.18.26/install)"
RUN sh -c "$(curl -sSfL https://release.anza.xyz/v2.2.1/install)"
#v2.0.18
ENV PATH="/root/.local/share/solana/install/active_release/bin:$PATH"

#Install Anchor version Manager(AVM)
# RUN cargo install --git https://github.com/coral-xyz/anchor avm --force
# RUN avm install 0.31.1
# RUN avm use 0.31.1

RUN cargo install --git https://github.com/coral-xyz/anchor anchor-cli --tag v0.31.1 --locked

# Set the working directory inside the container
WORKDIR /usr/src/myapp

# Copy entrypoint
COPY entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# Copy wallet key to container
COPY wallet_key.json /root/.config/solana/id.json

# Use entrypoint script
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]

CMD ["sleep", "infinity"]