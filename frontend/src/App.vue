<template>
  <div id="app">
    <header>
      <h1>Test Data Generator</h1>
      <p class="subtitle">Upload CSV files or generate sample data</p>
    </header>

    <div class="card">
      <FileUpload
        @file-uploaded="handleFileUpload"
        @upload-error="handleUploadError"
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
</template>

<script setup lang="ts">
import { ref } from 'vue'
import FileUpload from './components/FileUpload.vue'
import DataTable from './components/DataTable.vue'
import DataGenerator from './components/DataGenerator.vue'
import type { CsvData, Message, ApiResponse } from './types'

const csvData = ref<CsvData | null>(null)
const uploadMessage = ref<Message | null>(null)
const isLoading = ref<boolean>(false)

const handleFileUpload = (data: ApiResponse) => {
  csvData.value = data.data
  uploadMessage.value = {
    type: 'success',
    text: data.message
  }
  isLoading.value = false

  setTimeout(() => {
    uploadMessage.value = null
  }, 5000)
}

const handleUploadError = (error: string) => {
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
</script>
