# Build Instructions

Quick reference guide for building and running the Intent Segregation frontend.

## Prerequisites

- Node.js >= 18.x
- npm >= 9.x

## Quick Start

```bash
# Navigate to frontend directory
cd /home/user/Intent-Segregation-Cybersecurity-Architecture-for-AI/frontend

# Install dependencies (first time only)
npm install

# Create environment file
cp .env.example .env

# Start development server
npm run dev
```

Access the application at: **http://localhost:3000**

## Development Commands

```bash
# Start dev server with hot reload
npm run dev

# Run linter
npm run lint

# Build for production
npm run build

# Preview production build
npm run preview
```

## Environment Configuration

Edit `.env` file:

```env
# API Backend URL
VITE_API_URL=http://localhost:8080/api
```

## Production Build

```bash
# Create optimized build
npm run build

# Output will be in dist/ directory
# Total size: ~308KB (gzipped: ~97KB)

# Test the production build
npm run preview
```

## Deployment

### Option 1: Static Hosting (Netlify, Vercel, etc.)

```bash
# Build the project
npm run build

# Deploy the dist/ folder to your hosting service
```

### Option 2: Docker

```dockerfile
# Example Dockerfile
FROM node:18-alpine as build
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### Option 3: Traditional Web Server

Serve the `dist/` directory with any web server (nginx, Apache, etc.)

**Nginx Configuration:**
```nginx
server {
    listen 80;
    root /path/to/dist;
    index index.html;

    location / {
        try_files $uri $uri/ /index.html;
    }
}
```

## Project Structure

```
frontend/
├── src/
│   ├── api/              # API client
│   ├── components/       # Reusable UI components
│   ├── pages/           # Page components
│   ├── types/           # TypeScript types
│   ├── utils/           # Utility functions
│   └── App.tsx          # Main app with routing
├── dist/                # Production build output
├── node_modules/        # Dependencies
└── package.json         # Project configuration
```

## Pages

- `/` - Query processing interface
- `/approval/:id` - Supervisor approval page
- `/audit` - Audit log viewer

## Troubleshooting

### Port 3000 already in use

```bash
# Kill process using port 3000
lsof -ti:3000 | xargs kill -9

# Or change port in vite.config.ts
```

### Build fails with TypeScript errors

```bash
# Clear cache and rebuild
rm -rf node_modules dist
npm install
npm run build
```

### API connection refused

1. Ensure backend is running on port 8080
2. Check `VITE_API_URL` in `.env`
3. Verify CORS is enabled on backend

### Styling not working

```bash
# Rebuild Tailwind CSS
npm run build
```

## Performance

- **Bundle Size**: 308KB total
  - Main bundle: 244KB (77.8KB gzipped)
  - React vendor: 43.5KB (15.7KB gzipped)
  - CSS: 16.4KB (3.6KB gzipped)

## Dependencies

All dependencies are installed automatically with `npm install`.

Key packages:
- react: ^19.2.0
- react-router-dom: ^7.9.6
- axios: ^1.13.2
- tailwindcss: ^3.4.18
- vite: ^7.2.4

## Support

- README.md - Comprehensive user guide
- IMPLEMENTATION.md - Technical implementation details
- Source code comments - Inline documentation
