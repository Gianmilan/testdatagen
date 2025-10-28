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
    - **Option A:** Drag and drop a CSV file or click to upload
    - **Option B:** Generate sample data with custom row count (1-1000 rows)
    - View your data in an interactive table

### CLI Mode

```bash
# Parse a CSV file
cargo run path/to/file.csv

# Start web server on custom port
cargo run -- --serve --port 8080
```

## Features

### 📊 Data Management

- **CSV Upload**: Drag-and-drop or click to upload CSV files
- **Data Generation**: Create placeholder data without files
    - Customizable row count (1-1000)
    - Quick presets (10, 20, 50, 100)
    - Random data: ID, Name, Email, Age, City

### User Interface

- Modern Vue 3 + TypeScript web interface
- Responsive design with dark mode support
- Real-time data visualization
- Interactive statistics dashboard
- Professional UI with smooth animations

### Backend

- Rust with Actix-web framework
- Modular API architecture
- CORS-enabled for development
- Type-safe JSON responses
- Both CLI and web server modes

### Developer Experience

- Full TypeScript type safety
- Shared type definitions
- Hot module reloading (HMR)
- Type checking before build

## 🛠Tech Stack

**Backend:** Rust, Actix-web, CSV parser, Rand
**Frontend:** Vue 3, TypeScript, Vite, Axios
**Architecture:** RESTful API, Composition API, Modular handlers

## API Endpoints

| Method | Endpoint        | Description        |
|--------|-----------------|--------------------|
| `GET`  | `/api/health`   | Health check       |
| `POST` | `/api/upload`   | Upload CSV file    |
| `POST` | `/api/generate` | Generate test data |

### Generate Data Example

```bash
curl -X POST http://localhost:8080/api/generate \
  -H "Content-Type: application/json" \
  -d '{"row_count": 50}'
```

## 📦 Build & Deploy

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
src/api/handlers/     # Modular API handlers
frontend/src/
  ├── components/     # Vue components
  ├── types/          # TypeScript definitions
  └── main.ts         # App entry point
```

## License

MIT

## Author

GS
