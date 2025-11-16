# BUILD STAGE

FROM maven:3.9-eclipse-temurin-24 AS builder

LABEL maintainer="rshare-ronakgh97"
LABEL stage="builder"

WORKDIR /build

# Copy Maven wrapper and pom.xml first for better layer caching
COPY .mvn/ .mvn/
COPY mvnw pom.xml ./

# Download dependencies (cached if pom.xml hasn't changed!!)
RUN mvn dependency:go-offline -B

# Copy source code
COPY src ./src

# Build the application
RUN mvn clean package -DskipTests -B

# Verify the JAR was created
RUN ls -lh target/*.jar

# RUNTIME STAGE
FROM eclipse-temurin:24-jre-alpine

LABEL maintainer="rshare-ronakgh97"
LABEL description="R-Share Relay Server - Secure P2P File Transfer"
LABEL version="1.0.0"

# Install dependencies for health checks and debugging
RUN apk add --no-cache \
    curl \
    bash \
    tzdata \
    netcat-openbsd

# Create non-root user for security
RUN addgroup -g 1001 rshare && \
    adduser -D -u 1001 -G rshare rshare

# Create application directory
WORKDIR /app

# Copy JAR from builder stage
COPY --from=builder /build/target/*.jar app.jar

# Create directories for logs and temp files
RUN mkdir -p /app/logs /app/temp && \
    chown -R rshare:rshare /app

# Switch to non-root user
USER rshare

ARG HTTP_PORT=8080
ARG SOCKET_PORT=10000

ENV HTTP_PORT=${HTTP_PORT}
ENV SOCKET_PORT=${SOCKET_PORT}

# Expose ports
# 8080: HTTP API for session handshake
# 10000: TCP socket for file transfers
EXPOSE ${HTTP_PORT} ${SOCKET_PORT}

# Health check configuration
# Checks every 30 seconds
HEALTHCHECK  --interval=30s --timeout=3s --start-period=30s --retries=3 \
    CMD curl -fs http://localhost:${HTTP_PORT}/actuator/health >/dev/null && \
    nc -z localhost ${SOCKET_PORT} || exit 1

# JVM Configuration for containers
# - UseContainerSupport: Detect container memory limits
# - MaxRAMPercentage: Use up to 75% of available memory
# - G1GC: Low-latency garbage collector
ENV JAVA_OPTS="-XX:+UseContainerSupport \
    -XX:MaxRAMPercentage=75.0 \
    -XX:+UseG1GC \
    -XX:MaxGCPauseMillis=100 \
    -Djava.security.egd=file:/dev/./urandom \
    -Djava.io.tmpdir=/app/temp"

# Spring Boot configuration
# ENV SPRING_PROFILES_ACTIVE=prod / default is used if not set

# Run the application
ENTRYPOINT ["sh", "-c", "java $JAVA_OPTS -jar app.jar"]

# BUILD INSTRUCTIONS
#
# Build the image:
#   docker build -t rshare-server:latest ./server
#
# Run the container:
#   docker run -d -p 8080:8080 -p 10000:10000 --name rshare rshare-server:latest
#
# View logs:
#   docker logs -f rshare
#
# Stop container:
#   docker stop rshare
#