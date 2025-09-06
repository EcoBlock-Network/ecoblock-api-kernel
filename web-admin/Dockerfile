# Multi-stage Dockerfile for building Vite React app and serving with nginx

FROM node:18-alpine AS build
WORKDIR /app

COPY package.json package-lock.json ./
COPY tsconfig.json vite.config.ts postcss.config.cjs ./
COPY src ./src
COPY index.html ./
COPY . .

RUN npm ci --silent
RUN npm run build

FROM nginx:stable-alpine
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
