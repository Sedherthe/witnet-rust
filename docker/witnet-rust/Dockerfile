FROM --platform=$TARGETPLATFORM ubuntu:disco

# Install basic environment dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl

# Clean up apt packages so the docker image is as compact as possible
RUN apt-get clean && apt-get autoremove

# Set needed environment variables
ENV RUST_BACKTRACE=1
ARG WITNET_VERSION

# Copy context and cd into it
COPY / /
WORKDIR /

# Expose server ports
EXPOSE 21337
EXPOSE 21338
EXPOSE 11212

# Run the install script
RUN ["chmod", "+x", "./runner.sh"]
RUN ["./runner.sh"]

# Set compilation entry point (always gets executed)
ENTRYPOINT ["./witnet"]

# Set default command (can be overriden)
CMD ["node", "server"]
