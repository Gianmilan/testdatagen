<template>
  <div class="generator-container">
    <div class="generator-header">
      <h3>Generate Sample Data</h3>
      <p class="description">Create placeholder data without uploading a file</p>
    </div>

    <div class="generator-controls">
      <div class="control-group">
        <label for="row-count">Number of Rows</label>
        <div class="input-with-buttons">
          <button
              @click="decrementRows"
              :disabled="isGenerating || rowCount <= 1"
              class="btn-stepper"
          >
            -
          </button>
          <input
              id="row-count"
              type="number"
              v-model.number="rowCount"
              :disabled="isGenerating"
              min="1"
              max="1000"
              class="row-input"
          />
          <button
              @click="incrementRows"
              :disabled="isGenerating || rowCount >= 1000"
              class="btn-stepper"
          >
            +
          </button>
        </div>
      </div>

      <div class="preset-buttons">
        <button
            v-for="preset in presets"
            :key="preset"
            @click="rowCount = preset"
            :disabled="isGenerating"
            :class="['btn-preset', { active: rowCount === preset }]"
        >
          {{ preset }}
        </button>
      </div>
    </div>

    <button
        @click="handleGenerate"
        :disabled="isGenerating"
        class="btn-generate"
    >
      <span v-if="!isGenerating">
        Generate {{ rowCount }} Row{{ rowCount !== 1 ? 's' : '' }}
      </span>
      <span v-else class="generating">
        <div class="spinner-small"></div>
        Generating...
      </span>
    </button>

    <div class="info-box">
      <div class="info-icon">ℹ️</div>
      <div class="info-text">
        Sample data includes: ID, Name, Email, Age, and City columns with randomized values.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {ref} from 'vue'
import axios, {AxiosError} from 'axios'
import type {ApiResponse, ErrorResponse} from '@/types'

const emit = defineEmits<{
  'data-generated': [data: ApiResponse]
  'generation-error': [error: string]
}>()

const rowCount = ref<number>(20)
const isGenerating = ref<boolean>(false)
const presets: number[] = [10, 20, 50, 100]

const incrementRows = () => {
  if (rowCount.value < 1000) {
    rowCount.value++
  }
}

const decrementRows = () => {
  if (rowCount.value > 1) {
    rowCount.value--
  }
}

const handleGenerate = async () => {
  isGenerating.value = true

  try {
    const response = await axios.post<ApiResponse>('/api/generate', {
      row_count: rowCount.value
    })

    emit('data-generated', response.data)
  } catch (error) {
    const axiosError = error as AxiosError<ErrorResponse>
    const errorMessage = axiosError.response?.data?.error || 'Failed to generate data'
    emit('generation-error', errorMessage)
  } finally {
    isGenerating.value = false
  }
}
</script>

<style scoped>
.generator-container {
  background: var(--surface-color, #f8f9fa);
  border: 2px dashed var(--border-color, #dee2e6);
  border-radius: 12px;
  padding: 1.5rem;
  transition: all 0.3s ease;
}

.generator-container:hover {
  border-color: var(--primary-color, #667eea);
}

.generator-header {
  text-align: center;
  margin-bottom: 1.5rem;
}

.generator-header h3 {
  margin: 0 0 0.5rem 0;
  color: var(--text-primary, #2c3e50);
  font-size: 1.25rem;
}

.description {
  margin: 0;
  color: var(--text-muted, #6c757d);
  font-size: 0.875rem;
}

.generator-controls {
  text-align: center;
  margin-bottom: 1.5rem;
}

.control-group {
  margin-bottom: 1rem;
}

.control-group label {
  display: block;
  margin-bottom: 1rem;
  font-weight: 600;
  color: var(--text-primary, #2c3e50);
  font-size: 0.875rem;
}

.input-with-buttons {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
  align-items: center;
}

.btn-stepper {
  width: 40px;
  height: 40px;
  border: 2px solid var(--border-color, #dee2e6);
  background: white;
  border-radius: 8px;
  font-size: 1.25rem;
  font-weight: bold;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--primary-color, #667eea);
}

.btn-stepper:hover:not(:disabled) {
  border-color: var(--primary-color, #667eea);
  background: var(--primary-light, #f0f2ff);
  transform: scale(1.05);
}

.btn-stepper:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.row-input {
  width: 100px;
  height: 40px;
  text-align: center;
  font-size: 1.125rem;
  font-weight: 600;
  border: 2px solid var(--border-color, #dee2e6);
  border-radius: 8px;
  transition: border-color 0.2s;
}

.row-input:focus {
  outline: none;
  border-color: var(--primary-color, #667eea);
}

.row-input:disabled {
  background: #f8f9fa;
  cursor: not-allowed;
}

.preset-buttons {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
  flex-wrap: wrap;
}

.btn-preset {
  padding: 0.5rem 1rem;
  border: 2px solid var(--border-color, #dee2e6);
  background: white;
  border-radius: 8px;
  font-size: 0.875rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  color: var(--text-primary, #2c3e50);
}

.btn-preset:hover:not(:disabled) {
  border-color: var(--primary-color, #667eea);
  color: var(--primary-color, #667eea);
  transform: translateY(-2px);
}

.btn-preset:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-generate {
  width: 100%;
  padding: 0.875rem;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 10px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
  box-shadow: 0 4px 6px rgba(102, 126, 234, 0.2);
  margin-bottom: 1rem;
}

.btn-generate:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 12px rgba(102, 126, 234, 0.3);
}

.btn-generate:disabled {
  opacity: 0.7;
  cursor: not-allowed;
  transform: none;
}

.btn-generate {
  margin-right: 0.5rem;
  font-size: 1.25rem;
}

.generating {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
}

.spinner-small {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.info-box {
  display: flex;
  gap: 0.75rem;
  padding: 0.75rem;
  background: #e7f3ff;
  border-left: 4px solid #2196f3;
  border-radius: 6px;
  font-size: 0.875rem;
  color: #0d47a1;
}

.info-icon {
  flex-shrink: 0;
  font-size: 1.25rem;
}

.info-text {
  line-height: 1.5;
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .generator-container {
    background: #2d3748;
    border-color: #4a5568;
  }

  .generator-header h3,
  .control-group label {
    color: #e2e8f0;
  }

  .description {
    color: #a0aec0;
  }

  .btn-stepper,
  .btn-preset,
  .row-input {
    background: #1a202c;
    border-color: #4a5568;
    color: #e2e8f0;
  }

  .btn-preset {
    background: #667eea;
    color: white;
  }

  .info-box {
    background: #2c5282;
    border-color: #4299e1;
    color: #bee3f8;
  }
}
</style>
