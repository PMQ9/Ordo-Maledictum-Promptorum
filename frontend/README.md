# Intent Segregation System - Frontend

Modern React SPA frontend for the Intent Segregation Cybersecurity Architecture for AI.

## Features

- **Query Processing Interface**: Submit and process queries through the security pipeline
- **Real-time Visualization**: View parsed intents, voting results, and comparison analysis
- **Approval System**: Review and approve/reject queries requiring supervisor intervention
- **Audit Logs**: Comprehensive audit trail with filtering and search capabilities
- **Responsive Design**: Mobile-friendly interface built with TailwindCSS

## Tech Stack

- **React 19** - Modern UI library
- **TypeScript** - Type-safe development
- **Vite** - Fast build tool and dev server
- **React Router** - Client-side routing
- **TailwindCSS** - Utility-first CSS framework
- **Axios** - HTTP client for API communication

## Project Structure

```
frontend/
├── src/
│   ├── api/              # API client and integration
│   │   └── client.ts     # Axios API client
│   ├── components/       # Reusable UI components
│   │   ├── Alert.tsx
│   │   ├── Badge.tsx
│   │   ├── Button.tsx
│   │   ├── Card.tsx
│   │   ├── IntentVisualization.tsx
│   │   ├── Layout.tsx
│   │   └── Loading.tsx
│   ├── pages/            # Page components
│   │   ├── QueryInterface.tsx
│   │   ├── ApprovalReview.tsx
│   │   └── AuditLogs.tsx
│   ├── types/            # TypeScript type definitions
│   │   └── index.ts
│   ├── utils/            # Utility functions
│   │   └── format.ts
│   ├── App.tsx           # Main app component with routing
│   ├── main.tsx          # Application entry point
│   └── index.css         # Global styles with Tailwind
├── public/               # Static assets
├── index.html            # HTML template
├── vite.config.ts        # Vite configuration
├── tailwind.config.js    # TailwindCSS configuration
├── tsconfig.json         # TypeScript configuration
└── package.json          # Dependencies and scripts
```

## Getting Started

### Prerequisites

- Node.js >= 18.x
- npm >= 9.x

### Installation

1. Navigate to the frontend directory:
```bash
cd frontend
```

2. Install dependencies:
```bash
npm install
```

3. Create environment configuration:
```bash
cp .env.example .env
```

4. Configure the API URL in `.env`:
```env
VITE_API_URL=http://localhost:8080/api
```

### Development

Start the development server:

```bash
npm run dev
```

The application will be available at `http://localhost:3000`

Features:
- Hot Module Replacement (HMR)
- API proxy to backend (configured in vite.config.ts)
- TypeScript type checking
- TailwindCSS with JIT compilation

### Building for Production

Create an optimized production build:

```bash
npm run build
```

The build output will be in the `dist/` directory.

Build optimizations:
- Code minification with Terser
- Tree shaking and dead code elimination
- Chunk splitting for better caching
- Console and debugger statements removed
- No source maps (configurable)

### Preview Production Build

Test the production build locally:

```bash
npm run preview
```

### Linting

Run ESLint to check code quality:

```bash
npm run lint
```

## Pages

### 1. Query Interface (`/`)

Main interface for submitting queries to the Intent Segregation system.

**Features:**
- User ID and query input form
- Real-time processing status
- Visual display of parsed intent
- Multi-model voting results
- Intent comparison analysis
- Final decision and execution results

**Usage:**
1. Enter your User ID
2. Type your query
3. Click "Submit Query"
4. View the processing results and security analysis

### 2. Approval Review (`/approval/:id`)

Supervisor interface for reviewing and approving/rejecting queries.

**Features:**
- Complete request details
- Intent visualization
- Voting and comparison results
- Approval/rejection with notes
- Real-time status updates

**Usage:**
1. Navigate to approval request (via link or direct URL)
2. Review the query and security analysis
3. Enter Supervisor ID
4. Add optional notes
5. Click "Approve" or "Reject"

### 3. Audit Logs (`/audit`)

Comprehensive audit trail viewer with filtering capabilities.

**Features:**
- Paginated log entries
- Filter by user ID, date range
- Expandable details for each entry
- Execution time metrics
- Complete metadata display

**Usage:**
1. View all audit logs in table format
2. Apply filters to narrow down results
3. Click "Details" to expand individual entries
4. Navigate through pages using pagination controls

## API Integration

The frontend communicates with the backend API using Axios. All API calls are centralized in `/src/api/client.ts`.

### Endpoints Used

- `GET /api/health` - Health check
- `POST /api/process` - Submit query for processing
- `GET /api/results/:id` - Get processing result
- `GET /api/approvals/pending` - Get pending approvals
- `GET /api/approvals/:id` - Get specific approval request
- `POST /api/approvals/:id` - Submit approval decision
- `GET /api/audit/logs` - Get audit logs with filters

### Error Handling

All API errors are intercepted and formatted into user-friendly messages. The application displays errors using the Alert component.

## Deployment

### Static Hosting

The built application can be deployed to any static hosting service:

- **Netlify**: Drag and drop the `dist` folder
- **Vercel**: Connect your repository and deploy
- **AWS S3 + CloudFront**: Upload `dist` to S3 and serve via CloudFront
- **GitHub Pages**: Use GitHub Actions to deploy `dist`

### Environment Variables

Set the following environment variables in your hosting platform:

```
VITE_API_URL=https://your-api-domain.com/api
```

## License

Part of the Intent Segregation Cybersecurity Architecture for AI project.
