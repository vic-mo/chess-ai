# Production Deployment Guide

This guide covers deploying the chess AI application to production with Google Cloud Run (backend) and Vercel (frontend).

## Architecture

- **Frontend**: React/Vite web app deployed to Vercel
- **Backend**: Rust WebSocket server deployed to Google Cloud Run
- **Communication**: WebSocket (wss:// in production)

## Prerequisites

- [ ] Google Cloud Platform account with billing enabled
- [ ] gcloud CLI installed and authenticated
- [ ] Vercel account
- [ ] Git repository connected to Vercel

## Pre-Deployment Checklist

### Frontend (apps/web)

- [x] All console logs removed (using logger utility)
- [x] TypeScript build succeeds without errors
- [x] Production environment template created (`.env.production.template`)
- [ ] Update `.env.production.local` with actual Cloud Run URL
- [ ] Test production build locally: `pnpm build && pnpm preview`

### Backend (apps/uci-server)

- [x] Rust unused variable warnings fixed
- [x] Release build succeeds: `cargo build --release --bin uci-server`
- [x] Dockerfile configured for Cloud Run
- [x] cloudbuild.yaml configured
- [ ] Test Docker build locally
- [ ] Choose region for deployment

## Backend Deployment (Google Cloud Run)

### 1. Setup Google Cloud Project

```bash
# Set project ID
export PROJECT_ID="your-project-id"
gcloud config set project $PROJECT_ID

# Enable required APIs
gcloud services enable cloudbuild.googleapis.com
gcloud services enable run.googleapis.com
gcloud services enable containerregistry.googleapis.com
```

### 2. Build and Deploy

```bash
# Submit build to Google Cloud Build
gcloud builds submit --config cloudbuild.yaml

# The cloudbuild.yaml handles:
# - Building Docker image
# - Pushing to Google Container Registry
# - Deploying to Cloud Run
```

### 3. Configure Cloud Run Service

After deployment, get the service URL:

```bash
gcloud run services describe chess-engine \
  --region us-central1 \
  --format 'value(status.url)'
```

The URL will be something like: `https://chess-engine-xxxxx-uc.a.run.app`

**Important**: Change `https://` to `wss://` for WebSocket connections.

### 4. Test Backend

```bash
# Test the WebSocket endpoint
wscat -c wss://your-cloud-run-url.app

# Send a test message
{"type":"analyze","id":"test-1","fen":"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1","limit":{"kind":"depth","depth":3}}
```

## Frontend Deployment (Vercel)

### 1. Configure Environment Variables

Create `apps/web/.env.production.local`:

```env
VITE_ENGINE_SERVER_URL=wss://your-cloud-run-url.app
```

Replace `your-cloud-run-url.app` with your actual Cloud Run service URL.

### 2. Deploy to Vercel

#### Option A: Using Vercel CLI

```bash
# Install Vercel CLI
npm install -g vercel

# Deploy
cd apps/web
vercel --prod
```

#### Option B: Using Git Integration

1. Push changes to your Git repository
2. Vercel will automatically deploy on push to main branch
3. Set environment variable in Vercel dashboard:
   - Name: `VITE_ENGINE_SERVER_URL`
   - Value: `wss://your-cloud-run-url.app`

### 3. Verify Deployment

Visit your Vercel URL and test:

- [ ] App loads without errors
- [ ] WebSocket connection establishes (check browser console)
- [ ] Can make moves and get engine responses
- [ ] No console logs in production build

## Configuration Files

### vercel.json

```json
{
  "buildCommand": "cd apps/web && pnpm install && pnpm build",
  "outputDirectory": "apps/web/dist",
  "installCommand": "pnpm install",
  "framework": null
}
```

### cloudbuild.yaml

```yaml
steps:
  - name: 'gcr.io/cloud-builders/docker'
    args:
      ['build', '-t', 'gcr.io/$PROJECT_ID/chess-engine', '-f', 'apps/uci-server/Dockerfile', '.']

  - name: 'gcr.io/cloud-builders/docker'
    args: ['push', 'gcr.io/$PROJECT_ID/chess-engine']

  - name: 'gcr.io/cloud-builders/gcloud'
    args:
      - 'run'
      - 'deploy'
      - 'chess-engine'
      - '--image=gcr.io/$PROJECT_ID/chess-engine'
      - '--region=us-central1'
      - '--platform=managed'
      - '--allow-unauthenticated'
      - '--port=8080'
      - '--memory=512Mi'
      - '--timeout=3600'

timeout: 3600s
```

## Local Testing

### Test Docker Build

```bash
# Build the Docker image
docker build -f apps/uci-server/Dockerfile -t chess-engine .

# Run locally
docker run -p 8080:8080 -e PORT=8080 chess-engine

# Test connection
wscat -c ws://localhost:8080
```

### Test Production Build

```bash
# Build frontend
cd apps/web
pnpm build

# Preview production build
pnpm preview

# Open http://localhost:4173
```

## Monitoring and Logs

### Cloud Run Logs

```bash
# View logs
gcloud run services logs read chess-engine --region us-central1

# Stream logs
gcloud run services logs tail chess-engine --region us-central1
```

### Vercel Logs

View logs in the Vercel dashboard:

- https://vercel.com/[your-team]/[your-project]/deployments

## Troubleshooting

### WebSocket Connection Fails

1. Verify Cloud Run service is running:

   ```bash
   gcloud run services list
   ```

2. Check Cloud Run allows unauthenticated access:

   ```bash
   gcloud run services add-iam-policy-binding chess-engine \
     --region=us-central1 \
     --member="allUsers" \
     --role="roles/run.invoker"
   ```

3. Verify environment variable in Vercel uses `wss://` (not `https://`)

### Frontend Shows Errors

1. Check browser console for errors
2. Verify `VITE_ENGINE_SERVER_URL` is set correctly in Vercel
3. Check Network tab to see WebSocket connection status
4. Verify Cloud Run service is accessible

### Build Failures

#### TypeScript Errors

```bash
pnpm build
# Fix any type errors reported
```

#### Rust Build Errors

```bash
cargo build --release --bin uci-server
# Fix any compilation errors
```

#### Docker Build Errors

```bash
docker build -f apps/uci-server/Dockerfile -t chess-engine .
# Check Dockerfile and dependencies
```

## Cost Optimization

### Cloud Run

- Uses pay-per-request pricing
- Scales to zero when idle
- Default: 512Mi memory, 3600s timeout
- Adjust in cloudbuild.yaml if needed

### Vercel

- Free tier includes:
  - Unlimited deployments
  - 100GB bandwidth
  - Automatic HTTPS

## Security Considerations

- [x] All user input validated (FEN strings, UCI moves)
- [x] WebSocket connections use wss:// in production
- [x] Cloud Run service uses Google-managed SSL
- [x] No sensitive data in environment variables
- [x] Console logs removed from production build

## Rollback

### Vercel

```bash
# List deployments
vercel ls

# Promote a previous deployment
vercel promote [deployment-url]
```

### Cloud Run

```bash
# List revisions
gcloud run revisions list --service chess-engine --region us-central1

# Route traffic to previous revision
gcloud run services update-traffic chess-engine \
  --region us-central1 \
  --to-revisions [REVISION_NAME]=100
```

## Support

For issues or questions:

- Check logs in Cloud Run and Vercel dashboards
- Review error messages in browser console
- Test backend independently using wscat
- Verify all environment variables are set correctly
