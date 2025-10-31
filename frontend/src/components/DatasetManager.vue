<template>
  <div class="dataset-manager">
    <div class="header">
      <h2>My Datasets</h2>
      <button class="btn btn-primary" @click="showCreateModal = true">
        + Create Dataset
      </button>
    </div>

    <div v-if="loading" class="loading">
      <div class="spinner"></div>
      <p>Loading datasets...</p>
    </div>

    <div v-else-if="error" class="alert alert-error">
      {{ error }}
    </div>

    <div v-else-if="datasets.length === 0" class="empty-state">
      <p>No datasets found. Create one to get started!</p>
    </div>

    <div v-else class="datasets-grid">
      <div v-for="dataset in datasets" :key="dataset.id" class="dataset-card">
        <div class="dataset-header">
          <h3>{{ dataset.name }}</h3>
          <span :class="['badge', `badge-${dataset.data_type}`]">
            {{ dataset.data_type }}
          </span>
        </div>

        <div class="dataset-info">
          <div class="info-item">
            <span class="label">Columns:</span>
            <span class="value">{{ dataset.column_count }}</span>
          </div>
          <div class="info-item">
            <span class="label">Created:</span>
            <span class="value">{{ formatDate(dataset.created_at) }}</span>
          </div>
          <div v-if="dataset.has_sample_data" class="info-item">
            <span class="label">Sample rows:</span>
            <span class="value">{{ dataset.row_count }}</span>
          </div>
        </div>

        <div class="dataset-actions">
          <button
            class="btn btn-primary btn-sm"
            @click="openGenerateModal(dataset)"
          >
            Generate Data
          </button>
          <button
            class="btn btn-danger btn-sm"
            @click="confirmDelete(dataset)"
          >
            Delete
          </button>
        </div>
      </div>
    </div>

    <!-- Generate Modal -->
    <div v-if="showGenerateModal" class="modal-overlay" @click.self="showGenerateModal = false">
      <div class="modal">
        <h3>Generate Data from "{{ selectedDataset?.name }}"</h3>
        <p class="modal-subtitle">How many rows would you like to generate?</p>

        <div class="row-count-control">
          <input
            v-model.number="rowCount"
            type="number"
            min="1"
            max="1000"
            class="input"
          />
          <div class="preset-buttons">
            <button class="btn btn-sm" @click="rowCount = 10">10</button>
            <button class="btn btn-sm" @click="rowCount = 20">20</button>
            <button class="btn btn-sm" @click="rowCount = 50">50</button>
            <button class="btn btn-sm" @click="rowCount = 100">100</button>
          </div>
        </div>

        <div class="modal-actions">
          <button class="btn btn-secondary" @click="showGenerateModal = false">
            Cancel
          </button>
          <button
            class="btn btn-primary"
            @click="generateData"
            :disabled="generating"
          >
            {{ generating ? 'Generating...' : 'Generate' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div v-if="showDeleteModal" class="modal-overlay" @click.self="showDeleteModal = false">
      <div class="modal">
        <h3>Delete Dataset</h3>
        <p>Are you sure you want to delete "{{ selectedDataset?.name }}"?</p>
        <p class="text-muted">This action cannot be undone.</p>

        <div class="modal-actions">
          <button class="btn btn-secondary" @click="showDeleteModal = false">
            Cancel
          </button>
          <button
            class="btn btn-danger"
            @click="deleteDataset"
            :disabled="deleting"
          >
            {{ deleting ? 'Deleting...' : 'Delete' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import axios, { AxiosError } from 'axios'
import type { Dataset, GenerateFromDatasetRequest, GenerateFromDatasetResponse, ErrorResponse } from '../types'

const emit = defineEmits<{
  'data-generated': [data: GenerateFromDatasetResponse]
  'generation-error': [error: string]
}>()

const datasets = ref<Dataset[]>([])
const loading = ref<boolean>(true)
const error = ref<string | null>(null)

const showGenerateModal = ref<boolean>(false)
const showDeleteModal = ref<boolean>(false)
const showCreateModal = ref<boolean>(false)
const selectedDataset = ref<Dataset | null>(null)
const rowCount = ref<number>(20)
const generating = ref<boolean>(false)
const deleting = ref<boolean>(false)

onMounted(async () => {
  await fetchDatasets()
})

const fetchDatasets = async () => {
  loading.value = true
  error.value = null

  try {
    const response = await axios.get<Dataset[]>('/api/datasets')
    datasets.value = response.data
  } catch (err) {
    const axiosError = err as AxiosError<ErrorResponse>
    error.value = axiosError.response?.data?.error || 'Failed to load datasets'
  } finally {
    loading.value = false
  }
}

const openGenerateModal = (dataset: Dataset) => {
  selectedDataset.value = dataset
  rowCount.value = 20
  showGenerateModal.value = true
}

const generateData = async () => {
  if (!selectedDataset.value || rowCount.value < 1 || rowCount.value > 1000) {
    return
  }

  generating.value = true

  try {
    const request: GenerateFromDatasetRequest = {
      row_count: rowCount.value
    }

    const response = await axios.post<GenerateFromDatasetResponse>(
      `/api/datasets/${selectedDataset.value.id}/generate`,
      request
    )

    emit('data-generated', response.data)
    showGenerateModal.value = false
  } catch (err) {
    const axiosError = err as AxiosError<ErrorResponse>
    const errorMessage = axiosError.response?.data?.error || 'Failed to generate data'
    emit('generation-error', errorMessage)
  } finally {
    generating.value = false
  }
}

const confirmDelete = (dataset: Dataset) => {
  selectedDataset.value = dataset
  showDeleteModal.value = true
}

const deleteDataset = async () => {
  if (!selectedDataset.value) return

  deleting.value = true

  try {
    await axios.delete(`/api/datasets/${selectedDataset.value.id}`)
    showDeleteModal.value = false
    await fetchDatasets()
  } catch (err) {
    const axiosError = err as AxiosError<ErrorResponse>
    error.value = axiosError.response?.data?.error || 'Failed to delete dataset'
  } finally {
    deleting.value = false
  }
}

const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  })
}
</script>

<style scoped>
.dataset-manager {
  padding: 1rem 0;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 2rem;
}

.header h2 {
  font-size: 1.5rem;
  font-weight: 600;
}

.datasets-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 1.5rem;
}

.dataset-card {
  background: white;
  border-radius: 0.5rem;
  padding: 1.5rem;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: box-shadow 0.2s ease;
}

.dataset-card:hover {
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
}

.dataset-header {
  display: flex;
  justify-content: space-between;
  align-items: start;
  margin-bottom: 1rem;
}

.dataset-header h3 {
  font-size: 1.125rem;
  font-weight: 600;
  margin: 0;
  flex: 1;
}

.badge {
  padding: 0.25rem 0.75rem;
  border-radius: 9999px;
  font-size: 0.75rem;
  font-weight: 500;
  text-transform: capitalize;
}

.badge-uploaded {
  background: #dbeafe;
  color: #1e40af;
}

.badge-custom {
  background: #fce7f3;
  color: #9f1239;
}

.badge-generated {
  background: #dcfce7;
  color: #166534;
}

.dataset-info {
  margin-bottom: 1rem;
}

.info-item {
  display: flex;
  justify-content: space-between;
  padding: 0.5rem 0;
  border-bottom: 1px solid var(--border-color);
}

.info-item:last-child {
  border-bottom: none;
}

.info-item .label {
  color: var(--text-muted);
  font-size: 0.875rem;
}

.info-item .value {
  font-weight: 500;
  font-size: 0.875rem;
}

.dataset-actions {
  display: flex;
  gap: 0.5rem;
  margin-top: 1rem;
}

.empty-state {
  text-align: center;
  padding: 3rem;
  color: var(--text-muted);
}

.loading {
  text-align: center;
  padding: 3rem;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: white;
  border-radius: 0.5rem;
  padding: 2rem;
  max-width: 500px;
  width: 90%;
  box-shadow: 0 20px 25px rgba(0, 0, 0, 0.15);
}

.modal h3 {
  margin-top: 0;
  margin-bottom: 0.5rem;
}

.modal-subtitle {
  color: var(--text-muted);
  margin-bottom: 1.5rem;
}

.row-count-control {
  margin-bottom: 1.5rem;
}

.row-count-control .input {
  width: 100%;
  margin-bottom: 1rem;
}

.preset-buttons {
  display: flex;
  gap: 0.5rem;
}

.modal-actions {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
}
</style>
