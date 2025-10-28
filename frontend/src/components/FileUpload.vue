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
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import axios, { AxiosError } from 'axios'
import type { ApiResponse, ErrorResponse } from '../types'

interface Props {
  isLoading: boolean
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'file-uploaded': [data: ApiResponse]
  'upload-error': [error: string]
}>()

const fileInput = ref<HTMLInputElement | null>(null)
const selectedFile = ref<File | null>(null)
const isDragging = ref<boolean>(false)

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
    emit('upload-error', 'Please select a CSV file')
    return
  }

  selectedFile.value = file
  await uploadFile(file)
}

const uploadFile = async (file: File) => {
  const formData = new FormData()
  formData.append('file', file)

  try {
    const response = await axios.post<ApiResponse>('/api/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data'
      }
    })

    emit('file-uploaded', response.data)
  } catch (error) {
    const axiosError = error as AxiosError<ErrorResponse>
    const errorMessage = axiosError.response?.data?.error || 'Failed to upload file'
    emit('upload-error', errorMessage)
  }
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
</style>
