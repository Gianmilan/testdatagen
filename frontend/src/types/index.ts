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
