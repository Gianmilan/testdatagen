# Test Data Generator

A full-stack TypeScript and Rust application for CSV file processing and test data generation.

## Quick Start

### Web Interface (Recommended)

1. **Start the backend:**
   ```bash
   cargo run
   ```
   Backend starts on http://localhost:8080

2. **Start the frontend** (in a new terminal):
   ```bash
   cd frontend
   npm install    # first time only
   npm run dev
   ```
   Frontend starts on http://localhost:3000

3. **Use the application:**
    - Open http://localhost:3000 in your browser

    **Generator Tab:**
    - **Option A:** Upload a CSV file (drag-and-drop or click)
      - Automatically saves the schema as a dataset (enabled by default)
      - Optionally save sample data for pattern learning
      - Optionally specify column types manually
    - **Option B:** Generate sample data with custom row count (1-1000 rows)

    **My Datasets Tab:**
    - View all saved dataset schemas
    - Generate data from any saved dataset
    - Delete datasets you no longer need

### CLI Mode

```bash
# Parse a CSV file
cargo run path/to/file.csv

# Start web server on custom port
cargo run -- --serve --port 8080
```

## Features

### 📊 Data Management

- **CSV Upload & Schema Saving**
  - Drag-and-drop or click to upload CSV files
  - Automatically save CSV schema as reusable dataset
  - Auto-generated dataset name from filename
  - Optional: Save sample data (up to 100 rows) for pattern learning
  - Optional: Manually specify column types with auto-detection fallback

- **Dataset Management**
  - View all saved datasets in organized cards
  - Three dataset types: `uploaded`, `custom`, `generated`
  - Generate test data from any saved dataset schema
  - Customizable row count (1-1000) with quick presets
  - Delete datasets with confirmation
  - Track sample data availability per dataset

- **Intelligent Data Generation**
  - **FlexibleGenerator**: Auto-detects column types from header names
  - **11 supported data types**: ID, Name, Email, Age, City, Country, Phone, Date, Money, Text (auto-detect fallback)
  - Random realistic data generation
  - Reusable schemas for consistent test data
  - Quick presets (10, 20, 50, 100 rows)

### User Interface

- **Modern Vue 3 + TypeScript web interface**
- **Two-tab navigation**: Generator | My Datasets
- **Generator Tab**:
  - CSV upload with save options panel
  - Sample data generator
  - Real-time data visualization
- **My Datasets Tab**:
  - Dataset cards with type badges
  - Generate modal with row count controls
  - Delete confirmation dialogs
- **Responsive design** with professional styling
- **Interactive statistics dashboard**
- **Type-safe** with shared TypeScript definitions

### Backend

- Rust with Actix-web framework
- Modular API architecture with organized handlers
- CORS-enabled for development
- Type-safe JSON responses
- Both CLI and web server modes
- SQLite database for data persistence
- Trait-based generator system
- Structured logging

### Developer Experience

- Full TypeScript type safety
- Shared type definitions
- Hot module reloading (HMR)
- Type checking before build
- Environment variable configuration

## Tech Stack

**Backend:** Rust, Actix-web, SQLx (SQLite), CSV parser, Rand, Chrono
**Frontend:** Vue 3, TypeScript, Vite, Axios
**Database:** SQLite with async support
**Architecture:** RESTful API, Composition API, Modular handlers, Trait-based generators

## API Endpoints

### Core Endpoints

| Method   | Endpoint                    | Description                      |
|----------|-----------------------------|----------------------------------|
| `GET`    | `/api/health`               | Health check                     |
| `POST`   | `/api/upload`               | Upload and parse CSV file        |
| `POST`   | `/api/generate`             | Generate test data               |
| `POST`   | `/api/extract-headers`      | Extract CSV headers only         |

### Dataset Management Endpoints

| Method   | Endpoint                    | Description                      |
|----------|-----------------------------|----------------------------------|
| `GET`    | `/api/datasets`             | List all saved datasets          |
| `POST`   | `/api/datasets`             | Save a new dataset schema        |
| `GET`    | `/api/datasets/{id}`        | Get single dataset with data     |
| `DELETE` | `/api/datasets/{id}`        | Delete a dataset                 |
| `POST`   | `/api/datasets/{id}/generate` | Generate data from dataset     |

### API Examples

**Generate test data:**
```bash
curl -X POST http://localhost:8080/api/generate \
  -H "Content-Type: application/json" \
  -d '{"row_count": 50}'
```

**Save a dataset schema:**
```bash
curl -X POST http://localhost:8080/api/datasets \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Customer Schema",
    "headers": ["id", "name", "email", "age"],
    "data_type": "uploaded"
  }'
```

**Generate from saved dataset:**
```bash
curl -X POST http://localhost:8080/api/datasets/1/generate \
  -H "Content-Type: application/json" \
  -d '{"row_count": 100}'
```

## Build & Deploy

### Development

```bash
# Backend
cargo run

# Frontend
cd frontend
npm run dev
```

### Production

```bash
# Build backend
cargo build --release

# Build frontend (includes type checking)
cd frontend
npm run build

# Type check only
npm run type-check
```

## 🏗Project Structure

```
src/
  ├── api/handlers/
  │   ├── health.rs         # Health check endpoint
  │   ├── upload.rs         # CSV upload handler
  │   ├── generate.rs       # Data generation handler
  │   ├── extract_headers.rs# Header extraction
  │   └── datasets.rs       # Dataset CRUD + generation (NEW)
  ├── db/
  │   ├── models.rs         # Dataset, SaveDatasetRequest, etc.
  │   └── operations.rs     # Database CRUD operations
  ├── generators/
  │   └── flexible.rs       # FlexibleGenerator (auto-detect types)
  ├── multipart.rs          # File upload utilities
  └── main.rs               # Application entry point

frontend/src/
  ├── components/
  │   ├── FileUpload.vue    # Upload + save options (ENHANCED)
  │   ├── DataGenerator.vue # Sample data generator
  │   ├── DataTable.vue     # Data visualization
  │   └── DatasetManager.vue# Dataset management page (NEW)
  ├── types/
  │   └── index.ts          # Shared TypeScript interfaces
  └── main.ts               # App entry point

migrations/
  ├── 20251029124315_create_datasets.sql    # Initial tables
  └── 20251030144500_add_dataset_columns.sql # Add column_types, has_sample_data
```

## Troubleshooting

### Port Already in Use Error

If you see "address already in use" when starting the server:

```bash
# Find the process using port 8080
lsof -i :8080

# Kill it (replace PID with the number from above)
kill <PID>

# Or force kill if needed
kill -9 <PID>

# Quick one-liner to kill everything on port 8080
lsof -ti :8080 | xargs kill
```

### Frontend Can't Connect to Backend

1. Verify backend is running: `http://localhost:8080/api/health`
2. Check that both servers are on correct ports
3. Look for CORS errors in browser console

### Build Issues

```bash
# Clean and rebuild Rust
cargo clean && cargo build

# Check TypeScript types
cd frontend && npm run type-check
```

For more detailed troubleshooting, see [CLAUDE.md](CLAUDE.md).

## License

MIT

## Author

GS
