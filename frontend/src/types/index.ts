// Shared type definitions for the application

export interface CsvData {
  headers: string[]
  rows: string[][]
}

export interface ApiResponse {
  data: CsvData
  message: string
}

export interface ErrorResponse {
  error: string
}

export interface Message {
  type: 'success' | 'error'
  text: string
}

// New types for header extraction flow
export interface HeadersResponse {
  headers: string[]
  message: string
}

export interface GenerateRequest {
  row_count?: number
  save?: boolean
  headers?: string[]  // Custom headers for generation
}

// Dataset management types
export interface Dataset {
  id: number
  name: string
  created_at: string
  row_count: number
  column_count: number
  data_type: 'uploaded' | 'custom' | 'generated'
  has_sample_data: boolean
  column_types?: string  // JSON string of column type mappings
}

export interface SaveDatasetRequest {
  name: string
  headers: string[]
  data_type: 'uploaded' | 'custom' | 'generated'
  column_types?: Record<string, string>
  sample_data?: string[][]
}

export interface GenerateFromDatasetRequest {
  row_count: number
}

export interface GenerateFromDatasetResponse {
  data: CsvData
  message: string
}

export interface SaveDatasetResponse {
  id: number
  message: string
}

export interface ColumnTypeOption {
  value: string
  label: string
}
