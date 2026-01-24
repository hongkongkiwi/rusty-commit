# Runtime-only image: copy prebuilt musl binaries
FROM --platform=$TARGETPLATFORM alpine:3.19 AS runtime

ARG TARGETARCH

# Install runtime dependencies
RUN apk add --no-cache openssl ca-certificates

# Create non-root user
RUN addgroup -g 1000 appgroup && \
    adduser -u 1000 -G appgroup -s /bin/sh -D appuser

# Copy binary for the current target architecture
COPY docker-bin/rusty-commit-${TARGETARCH} /usr/local/bin/rusty-commit

# Create config directory
RUN mkdir /config && chown appuser:appgroup /config

# Switch to non-root user
USER appuser

# Default command
ENTRYPOINT ["rusty-commit"]
CMD ["stdio"]
