<template>
  <div>
    <div
      :class="['upload-zone', { 'drag-over': isDragging }]"
      @click="triggerFileInput"
      @dragover.prevent="handleDragOver"
      @dragleave.prevent="handleDragLeave"
      @drop.prevent="handleDrop"
    >
      <div v-if="!isLoading">
        <div class="upload-icon">📁</div>
        <p><strong>Click to upload</strong> or drag and drop</p>
        <p class="text-muted">CSV files only</p>
      </div>
      <div v-else class="loading">
        <div class="spinner"></div>
        <p>Uploading and processing...</p>
      </div>
    </div>

    <input
      ref="fileInput"
      type="file"
      class="file-input"
      accept=".csv"
      @change="handleFileSelect"
    />

    <div v-if="selectedFile && !isLoading" class="file-info">
      Selected: {{ selectedFile.name }} ({{ formatFileSize(selectedFile.size) }})
    </div>

    <!-- Save Options Panel -->
    <div v-if="selectedFile && !isLoading" class="save-options">
      <div class="option-group">
        <label class="checkbox-label">
          <input type="checkbox" v-model="saveAsDataset" />
          <span>Save this schema as a dataset</span>
        </label>
      </div>

      <div v-if="saveAsDataset" class="nested-options">
        <div class="option-group">
          <label class="input-label">Dataset name:</label>
          <input
            type="text"
            v-model="datasetName"
            class="input"
            placeholder="Enter dataset name"
          />
        </div>

        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="saveSampleData" />
            <span>Save sample data for pattern analysis</span>
          </label>
          <p class="help-text">
            Saves up to 100 rows to help generate more realistic fake data
          </p>
        </div>

        <div class="option-group">
          <label class="checkbox-label">
            <input type="checkbox" v-model="manualTypes" />
            <span>Manually specify column types</span>
          </label>
          <div class="warning-box">
            <span class="warning-icon">⚠️</span>
            <p>FlexibleGenerator auto-detection is enabled by default and may make mistakes. Manual specification can improve accuracy.</p>
          </div>
        </div>

        <div v-if="manualTypes && extractedHeaders.length > 0" class="column-types">
          <p class="section-label">Column Types:</p>
          <div v-for="header in extractedHeaders" :key="header" class="column-type-row">
            <span class="column-name">{{ header }}</span>
            <select v-model="columnTypes[header]" class="type-select">
              <option v-for="option in COLUMN_TYPE_OPTIONS" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import axios, { AxiosError } from 'axios'
import type { HeadersResponse, ErrorResponse, SaveDatasetRequest, SaveDatasetResponse } from '../types'

interface Props {
  isLoading: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'headers-extracted': [headers: string[]]
  'extraction-error': [error: string]
}>()

const fileInput = ref<HTMLInputElement | null>(null)
const selectedFile = ref<File | null>(null)
const isDragging = ref<boolean>(false)

// Save dataset options
const saveAsDataset = ref<boolean>(true)
const datasetName = ref<string>('')
const saveSampleData = ref<boolean>(false)
const manualTypes = ref<boolean>(false)
const extractedHeaders = ref<string[]>([])
const columnTypes = ref<Record<string, string>>({})

const COLUMN_TYPE_OPTIONS = [
  { value: 'auto', label: 'Auto Detect' },
  { value: 'id', label: 'ID' },
  { value: 'name', label: 'Name' },
  { value: 'email', label: 'Email' },
  { value: 'age', label: 'Age' },
  { value: 'city', label: 'City' },
  { value: 'country', label: 'Country' },
  { value: 'phone', label: 'Phone' },
  { value: 'date', label: 'Date' },
  { value: 'money', label: 'Money' },
  { value: 'text', label: 'Text' }
]

const triggerFileInput = () => {
  if (!props.isLoading) {
    fileInput.value?.click()
  }
}

const handleDragOver = () => {
  if (!props.isLoading) {
    isDragging.value = true
  }
}

const handleDragLeave = () => {
  isDragging.value = false
}

const handleDrop = (e: DragEvent) => {
  isDragging.value = false
  if (!props.isLoading && e.dataTransfer) {
    const files = e.dataTransfer.files
    if (files.length > 0) {
      processFile(files[0])
    }
  }
}

const handleFileSelect = (e: Event) => {
  const target = e.target as HTMLInputElement
  const files = target.files
  if (files && files.length > 0) {
    processFile(files[0])
  }
}

const processFile = async (file: File) => {
  if (!file.name.endsWith('.csv')) {
    emit('extraction-error', 'Please select a CSV file')
    return
  }

  selectedFile.value = file
  datasetName.value = generateDatasetName(file.name)
  await extractHeaders(file)
}

const extractHeaders = async (file: File) => {
  const formData = new FormData()
  formData.append('file', file)

  try {
    const response = await axios.post<HeadersResponse>('/api/extract-headers', formData, {
      headers: {
        'Content-Type': 'multipart/form-data'
      }
    })

    extractedHeaders.value = response.data.headers

    // Initialize column types if manual mode
    if (manualTypes.value) {
      columnTypes.value = {}
      response.data.headers.forEach(header => {
        columnTypes.value[header] = 'auto'
      })
    }

    // Save dataset if option is enabled
    if (saveAsDataset.value) {
      await saveDataset(response.data.headers, file)
    }

    emit('headers-extracted', response.data.headers)
  } catch (error) {
    const axiosError = error as AxiosError<ErrorResponse>
    const errorMessage = axiosError.response?.data?.error || 'Failed to extract headers'
    emit('extraction-error', errorMessage)
  }
}

const generateDatasetName = (filename: string): string => {
  return filename
    .replace('.csv', '')
    .replace(/[_-]/g, ' ')
    .replace(/\b\w/g, c => c.toUpperCase())
    .trim()
}

const saveDataset = async (headers: string[], file: File) => {
  try {
    const request: SaveDatasetRequest = {
      name: datasetName.value || generateDatasetName(file.name),
      headers: headers,
      data_type: 'uploaded',
      column_types: manualTypes.value ? columnTypes.value : undefined,
      sample_data: saveSampleData.value ? await readSampleData(file) : undefined
    }

    await axios.post<SaveDatasetResponse>('/api/datasets', request)
  } catch (error) {
    // Don't fail the whole process if saving fails, just log it
    console.error('Failed to save dataset:', error)
  }
}

const readSampleData = async (file: File): Promise<string[][]> => {
  // For now, we'll skip reading actual CSV data
  // In a full implementation, you'd parse the CSV and return the first 100 rows
  return []
}

const formatFileSize = (bytes: number) => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i]
}
</script>

<style scoped>
.text-muted {
  color: var(--text-muted);
  font-size: 0.875rem;
}

.save-options {
  margin-top: 1.5rem;
  padding: 1.5rem;
  background: var(--bg-color);
  border-radius: 0.5rem;
  border: 1px solid var(--border-color);
}

.option-group {
  margin-bottom: 1rem;
}

.option-group:last-child {
  margin-bottom: 0;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  cursor: pointer;
  font-size: 0.9375rem;
}

.checkbox-label input[type="checkbox"] {
  width: 1.125rem;
  height: 1.125rem;
  cursor: pointer;
}

.checkbox-label span {
  font-weight: 500;
}

.input-label {
  display: block;
  margin-bottom: 0.5rem;
  font-weight: 500;
  font-size: 0.875rem;
}

.input {
  width: 100%;
  padding: 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  font-size: 0.9375rem;
}

.input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.nested-options {
  margin-left: 1.75rem;
  margin-top: 1rem;
  padding-left: 1rem;
  border-left: 2px solid var(--border-color);
}

.help-text {
  margin: 0.5rem 0 0 1.75rem;
  font-size: 0.8125rem;
  color: var(--text-muted);
}

.warning-box {
  display: flex;
  gap: 0.75rem;
  margin: 0.75rem 0 0 1.75rem;
  padding: 0.75rem;
  background: #fef3c7;
  border: 1px solid #fcd34d;
  border-radius: 0.375rem;
}

.warning-icon {
  font-size: 1.25rem;
  flex-shrink: 0;
}

.warning-box p {
  margin: 0;
  font-size: 0.8125rem;
  color: #92400e;
  line-height: 1.5;
}

.column-types {
  margin-top: 1rem;
}

.section-label {
  font-weight: 500;
  font-size: 0.875rem;
  margin-bottom: 0.75rem;
}

.column-type-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.5rem;
  margin-bottom: 0.5rem;
  background: white;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
}

.column-name {
  font-weight: 500;
  font-size: 0.875rem;
  flex: 1;
}

.type-select {
  padding: 0.375rem 0.625rem;
  border: 1px solid var(--border-color);
  border-radius: 0.375rem;
  font-size: 0.875rem;
  cursor: pointer;
}

.type-select:focus {
  outline: none;
  border-color: var(--primary-color);
}
</style>
