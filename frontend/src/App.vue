<template>
  <div id="app">
    <header>
      <h1>Test Data Generator</h1>
      <p class="subtitle">Upload CSV files or generate sample data</p>

      <nav class="tabs">
        <button
          :class="['tab', { active: activeTab === 'generator' }]"
          @click="activeTab = 'generator'"
        >
          Generator
        </button>
        <button
          :class="['tab', { active: activeTab === 'datasets' }]"
          @click="activeTab = 'datasets'"
        >
          My Datasets
        </button>
      </nav>
    </header>

    <div v-if="activeTab === 'generator'">
      <div class="card">
        <FileUpload
          @headers-extracted="handleHeadersExtracted"
          @extraction-error="handleExtractionError"
          :is-loading="isLoading"
        />

        <div class="divider">
          <span class="divider-text">OR</span>
        </div>

        <DataGenerator
          @data-generated="handleDataGenerated"
          @generation-error="handleGenerationError"
        />

        <div v-if="uploadMessage" :class="['alert', uploadMessage.type === 'success' ? 'alert-success' : 'alert-error']">
          {{ uploadMessage.text }}
        </div>
      </div>

      <DataTable v-if="csvData" :data="csvData" />
    </div>

    <div v-if="activeTab === 'datasets'">
      <DatasetManager
        @data-generated="handleDatasetGenerated"
        @generation-error="handleGenerationError"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import axios, { AxiosError } from 'axios'
import FileUpload from './components/FileUpload.vue'
import DataTable from './components/DataTable.vue'
import DataGenerator from './components/DataGenerator.vue'
import DatasetManager from './components/DatasetManager.vue'
import type { CsvData, Message, ApiResponse, GenerateRequest, ErrorResponse, GenerateFromDatasetResponse } from './types'

const activeTab = ref<'generator' | 'datasets'>('generator')
const csvData = ref<CsvData | null>(null)
const uploadMessage = ref<Message | null>(null)
const isLoading = ref<boolean>(false)

/**
 * Handle headers extracted from uploaded CSV file
 * This is step 1 of the new flow: Extract headers from file
 * Next step: Generate data based on those headers
 */
const handleHeadersExtracted = async (headers: string[]) => {
  uploadMessage.value = {
    type: 'success',
    text: `Extracted ${headers.length} column${headers.length === 1 ? '' : 's'}. Generating data...`
  }

  // Step 2: Generate data using the extracted headers
  try {
    const generateRequest: GenerateRequest = {
      row_count: 20,  // Default row count
      headers: headers
    }

    const response = await axios.post<ApiResponse>('/api/generate', generateRequest)

    csvData.value = response.data.data
    uploadMessage.value = {
      type: 'success',
      text: response.data.message
    }
    isLoading.value = false

    setTimeout(() => {
      uploadMessage.value = null
    }, 5000)
  } catch (error) {
    const axiosError = error as AxiosError<ErrorResponse>
    const errorMessage = axiosError.response?.data?.error || 'Failed to generate data'
    handleExtractionError(errorMessage)
  }
}

const handleExtractionError = (error: string) => {
  uploadMessage.value = {
    type: 'error',
    text: error
  }
  isLoading.value = false

  setTimeout(() => {
    uploadMessage.value = null
  }, 5000)
}

const handleDataGenerated = (data: ApiResponse) => {
  csvData.value = data.data
  uploadMessage.value = {
    type: 'success',
    text: data.message
  }

  setTimeout(() => {
    uploadMessage.value = null
  }, 5000)
}

const handleGenerationError = (error: string) => {
  uploadMessage.value = {
    type: 'error',
    text: error
  }

  setTimeout(() => {
    uploadMessage.value = null
  }, 5000)
}

const handleDatasetGenerated = (data: GenerateFromDatasetResponse) => {
  csvData.value = data.data
  uploadMessage.value = {
    type: 'success',
    text: data.message
  }
  // Switch to generator tab to show the data
  activeTab.value = 'generator'

  setTimeout(() => {
    uploadMessage.value = null
  }, 5000)
}
</script>

<style scoped>
.tabs {
  display: flex;
  gap: 0.5rem;
  margin-top: 1.5rem;
  border-bottom: 2px solid var(--border-color);
}

.tab {
  padding: 0.75rem 1.5rem;
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--text-muted);
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  margin-bottom: -2px;
}

.tab:hover {
  color: var(--primary-color);
}

.tab.active {
  color: var(--primary-color);
  border-bottom-color: var(--primary-color);
}
</style>
