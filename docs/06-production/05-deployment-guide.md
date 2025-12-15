# 生产级部署指南

> **状态**: ✅ 生产就绪  
> **版本**: V2.0  
> **更新日期**: 2025-11-25

---

## 📋 目录

1. [Docker 容器化](#docker-容器化)
2. [Kubernetes 部署](#kubernetes-部署)
3. [CI/CD Pipeline](#cicd-pipeline)
4. [SSL/TLS 配置](#ssltls-配置)
5. [CDN 配置](#cdn-配置)
6. [灾难恢复](#灾难恢复)

---

## 🐳 Docker 容器化

### Dockerfile（多阶段构建）

```dockerfile
# IronForge/Dockerfile
# ====================
# Stage 1: Build
# ====================
FROM rust:1.75-slim as builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 安装 trunk (WASM 构建工具)
RUN cargo install --locked trunk

# 安装 wasm 目标
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# 复制依赖文件（利用缓存）
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY index.html ./
COPY Trunk.toml ./

# 构建生产版本
RUN trunk build --release

# ====================
# Stage 2: Runtime
# ====================
FROM nginx:alpine

# 复制 nginx 配置
COPY nginx.conf /etc/nginx/nginx.conf

# 复制构建产物
COPY --from=builder /app/dist /usr/share/nginx/html

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost/health || exit 1

EXPOSE 80 443

CMD ["nginx", "-g", "daemon off;"]
```

### Nginx 配置

```nginx
# nginx.conf
user nginx;
worker_processes auto;
error_log /var/log/nginx/error.log warn;
pid /var/run/nginx.pid;

events {
    worker_connections 1024;
}

http {
    include /etc/nginx/mime.types;
    default_type application/octet-stream;

    log_format main '$remote_addr - $remote_user [$time_local] "$request" '
                    '$status $body_bytes_sent "$http_referer" '
                    '"$http_user_agent" "$http_x_forwarded_for"';

    access_log /var/log/nginx/access.log main;

    sendfile on;
    tcp_nopush on;
    tcp_nodelay on;
    keepalive_timeout 65;
    types_hash_max_size 2048;

    # Gzip 压缩
    gzip on;
    gzip_vary on;
    gzip_proxied any;
    gzip_comp_level 6;
    gzip_types text/plain text/css text/xml text/javascript 
               application/json application/javascript application/xml+rss 
               application/wasm;

    # 安全头
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    
    # CSP
    add_header Content-Security-Policy "default-src 'self'; \
        script-src 'self' 'wasm-unsafe-eval'; \
        style-src 'self' 'unsafe-inline'; \
        img-src 'self' data: https:; \
        connect-src 'self' wss: https:; \
        font-src 'self' data:;" always;

    server {
        listen 80;
        server_name _;
        root /usr/share/nginx/html;
        index index.html;

        # SPA 路由支持
        location / {
            try_files $uri $uri/ /index.html;
        }

        # WASM 文件类型
        location ~* \.wasm$ {
            types {
                application/wasm wasm;
            }
            add_header Cache-Control "public, max-age=31536000, immutable";
        }

        # 静态资源缓存
        location ~* \.(js|css|png|jpg|jpeg|gif|ico|svg)$ {
            expires 1y;
            add_header Cache-Control "public, immutable";
        }

        # 健康检查端点
        location /health {
            access_log off;
            return 200 "healthy\n";
            add_header Content-Type text/plain;
        }

        # API 代理（避免 CORS）
        location /api/ {
            proxy_pass ${BACKEND_URL};
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }

        # WebSocket 代理
        location /ws/ {
            proxy_pass ${BACKEND_WS_URL};
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
            proxy_set_header Host $host;
        }
    }

    # HTTPS 重定向（生产环境）
    server {
        listen 443 ssl http2;
        server_name ${DOMAIN_NAME};

        ssl_certificate /etc/nginx/ssl/cert.pem;
        ssl_certificate_key /etc/nginx/ssl/key.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;
        ssl_prefer_server_ciphers on;
        ssl_session_cache shared:SSL:10m;
        ssl_session_timeout 10m;

        # HSTS
        add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;

        # ... 其他配置同上 ...
    }
}
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  ironforge-frontend:
    build:
      context: .
      dockerfile: Dockerfile
    image: ironforge/frontend:latest
    container_name: ironforge-frontend
    restart: unless-stopped
    ports:
      - "80:80"
      - "443:443"
    environment:
      - BACKEND_URL=https://api.ironforge.io
      - BACKEND_WS_URL=wss://api.ironforge.io
      - DOMAIN_NAME=app.ironforge.io
    volumes:
      - ./ssl:/etc/nginx/ssl:ro
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    networks:
      - ironforge-net
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  # Prometheus 监控
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    ports:
      - "9090:9090"
    networks:
      - ironforge-net
    restart: unless-stopped

  # Grafana 可视化
  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_ADMIN_PASSWORD}
      - GF_USERS_ALLOW_SIGN_UP=false
    ports:
      - "3000:3000"
    networks:
      - ironforge-net
    restart: unless-stopped

networks:
  ironforge-net:
    driver: bridge

volumes:
  prometheus_data:
  grafana_data:
```

---

## ☸️ Kubernetes 部署

### Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ironforge-frontend
  namespace: ironforge
  labels:
    app: ironforge-frontend
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: ironforge-frontend
  template:
    metadata:
      labels:
        app: ironforge-frontend
        version: v2.0.0
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      containers:
      - name: frontend
        image: ironforge/frontend:v2.0.0
        imagePullPolicy: IfNotPresent
        ports:
        - containerPort: 80
          name: http
        - containerPort: 443
          name: https
        env:
        - name: BACKEND_URL
          valueFrom:
            configMapKeyRef:
              name: ironforge-config
              key: backend_url
        - name: BACKEND_WS_URL
          valueFrom:
            configMapKeyRef:
              name: ironforge-config
              key: backend_ws_url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 80
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health
            port: 80
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
        volumeMounts:
        - name: ssl-certs
          mountPath: /etc/nginx/ssl
          readOnly: true
      volumes:
      - name: ssl-certs
        secret:
          secretName: ironforge-tls
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - ironforge-frontend
              topologyKey: kubernetes.io/hostname
```

### Service

```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: ironforge-frontend
  namespace: ironforge
  labels:
    app: ironforge-frontend
spec:
  type: ClusterIP
  ports:
  - port: 80
    targetPort: 80
    protocol: TCP
    name: http
  - port: 443
    targetPort: 443
    protocol: TCP
    name: https
  selector:
    app: ironforge-frontend
```

### Ingress

```yaml
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: ironforge-ingress
  namespace: ironforge
  annotations:
    kubernetes.io/ingress.class: "nginx"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
    nginx.ingress.kubernetes.io/rate-limit: "100"
spec:
  tls:
  - hosts:
    - app.ironforge.io
    secretName: ironforge-tls
  rules:
  - host: app.ironforge.io
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: ironforge-frontend
            port:
              number: 80
```

### HorizontalPodAutoscaler

```yaml
# k8s/hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: ironforge-frontend-hpa
  namespace: ironforge
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: ironforge-frontend
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 50
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
      - type: Pods
        value: 2
        periodSeconds: 15
      selectPolicy: Max
```

---

## 🔄 CI/CD Pipeline

### GitHub Actions

```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    branches: [main]
    tags: ['v*']

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ironforge/frontend

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
      
      - name: Install trunk
        run: cargo install trunk
      
      - name: Run tests
        run: cargo test --all-features
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Check formatting
        run: cargo fmt --check

  build:
    needs: test
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3
      
      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=sha
      
      - name: Build and push Docker image
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up kubectl
        uses: azure/setup-kubectl@v3
      
      - name: Configure kubectl
        run: |
          echo "${{ secrets.KUBECONFIG }}" | base64 -d > kubeconfig
          export KUBECONFIG=./kubeconfig
      
      - name: Deploy to Kubernetes
        run: |
          kubectl set image deployment/ironforge-frontend \
            frontend=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }} \
            -n ironforge
          
          kubectl rollout status deployment/ironforge-frontend -n ironforge
      
      - name: Notify Slack
        if: always()
        uses: 8398a7/action-slack@v3
        with:
          status: ${{ job.status }}
          webhook_url: ${{ secrets.SLACK_WEBHOOK }}
```

---

## 🔐 SSL/TLS 配置

### Let's Encrypt 自动证书

```yaml
# k8s/cert-manager.yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@ironforge.io
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
```

---

## 🌐 CDN 配置

### Cloudflare Workers

```javascript
// cloudflare-worker.js
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

async function handleRequest(request) {
  const url = new URL(request.url)
  
  // 缓存策略
  const cacheKey = new Request(url.toString(), request)
  const cache = caches.default
  
  // 检查缓存
  let response = await cache.match(cacheKey)
  
  if (!response) {
    response = await fetch(request)
    
    // 缓存静态资源
    if (url.pathname.match(/\.(js|css|wasm|png|jpg|svg)$/)) {
      response = new Response(response.body, response)
      response.headers.set('Cache-Control', 'public, max-age=31536000, immutable')
      event.waitUntil(cache.put(cacheKey, response.clone()))
    }
  }
  
  return response
}
```

---

## 🔄 灾难恢复

### 备份策略

```bash
#!/bin/bash
# backup.sh - 定期备份脚本

BACKUP_DIR="/backups/ironforge"
DATE=$(date +%Y%m%d_%H%M%S)

# 备份配置文件
tar -czf "$BACKUP_DIR/config_$DATE.tar.gz" \
    config.toml \
    .env \
    k8s/

# 备份 Docker 镜像
docker save ironforge/frontend:latest | gzip > "$BACKUP_DIR/frontend_$DATE.tar.gz"

# 清理旧备份（保留 30 天）
find "$BACKUP_DIR" -type f -mtime +30 -delete

echo "Backup completed: $DATE"
```

### 恢复流程

```bash
#!/bin/bash
# restore.sh - 灾难恢复脚本

BACKUP_FILE=$1

# 1. 恢复配置
tar -xzf "$BACKUP_FILE"

# 2. 恢复 Docker 镜像
docker load < frontend_backup.tar.gz

# 3. 重新部署
kubectl apply -f k8s/

# 4. 验证健康
kubectl rollout status deployment/ironforge-frontend -n ironforge
```

---

## 📚 部署检查清单

### 部署前

- [ ] 代码已通过所有测试
- [ ] 配置文件已更新（config.toml, .env）
- [ ] 密钥已更新为生产密钥
- [ ] SSL 证书已配置
- [ ] 数据库迁移已完成
- [ ] CDN 已配置
- [ ] 监控告警已设置
- [ ] 备份策略已验证

### 部署中

- [ ] 使用滚动更新（零停机）
- [ ] 监控错误日志
- [ ] 验证健康检查
- [ ] 确认性能指标正常

### 部署后

- [ ] 执行烟雾测试
- [ ] 验证关键功能
- [ ] 检查错误率
- [ ] 确认监控指标
- [ ] 通知团队部署完成
- [ ] 更新文档

---

## 🔗 相关文档

- [配置管理](./01-configuration-management.md)
- [监控配置](./04-monitoring-setup.md)
- [安全加固](./06-security-hardening.md)
