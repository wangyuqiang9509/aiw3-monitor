# 构建阶段
FROM rust:1.80 as builder

WORKDIR /build

# 复制依赖文件
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/benches ./benches
COPY backend/.sqlx ./.sqlx

# 构建依赖(利用Docker缓存)
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制源代码
COPY backend/src ./src

# 构建应用
ENV SQLX_OFFLINE=true
RUN touch src/main.rs && \
    cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /build/target/release/aiw3-monitor /app/

# 暴露Prometheus端口
EXPOSE 9090

CMD ["/app/aiw3-monitor"]

