<template>
  <div class="dataset-manager">
    <div class="header">
      <h2>My Datasets</h2>
      <button class="btn btn-primary" @click="openCreateForm">
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
              class="btn btn-secondary btn-sm"
              @click="openViewModal(dataset)"
              title="View details"
          >
            View
          </button>
          <button
              class="btn btn-primary btn-sm"
              @click="openGenerateModal(dataset)"
          >
            Generate
          </button>
          <button
              class="btn btn-secondary btn-sm"
              @click="openEditForm(dataset)"
              v-if="dataset.data_type === 'custom'"
              title="Edit dataset"
          >
            Edit
          </button>
          <button
              class="btn btn-secondary btn-sm"
              @click="duplicateDataset(dataset)"
              title="Duplicate dataset"
          >
            Duplicate
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

    <!-- View/Details Modal -->
    <div v-if="showViewModal" class="modal-overlay" @click.self="showViewModal = false">
      <div class="modal modal-large">
        <div class="modal-header">
          <h3>{{ selectedDataset?.name }}</h3>
          <button class="btn-close" @click="showViewModal = false">&times;</button>
        </div>

        <div v-if="loadingDetails" class="loading">
          <div class="spinner"></div>
          <p>Loading dataset details...</p>
        </div>

        <div v-else-if="datasetDetails" class="dataset-details">
          <div class="details-grid">
            <div class="detail-item">
              <span class="label">Type:</span>
              <span :class="['badge', `badge-${datasetDetails.dataset.data_type}`]">
                {{ datasetDetails.dataset.data_type }}
              </span>
            </div>
            <div class="detail-item">
              <span class="label">Columns:</span>
              <span class="value">{{ datasetDetails.dataset.column_count }}</span>
            </div>
            <div class="detail-item">
              <span class="label">Created:</span>
              <span class="value">{{ formatDate(datasetDetails.dataset.created_at) }}</span>
            </div>
            <div class="detail-item">
              <span class="label">Sample Data:</span>
              <span class="value">{{ datasetDetails.dataset.has_sample_data ? 'Yes' : 'No' }}</span>
            </div>
          </div>

          <div class="columns-preview">
            <h4>Columns ({{ datasetDetails.data.headers.length }})</h4>
            <div class="columns-tags">
              <span v-for="header in datasetDetails.data.headers" :key="header" class="column-tag">
                {{ header }}
              </span>
            </div>
          </div>

          <div v-if="datasetDetails.dataset.has_sample_data && datasetDetails.data.rows.length > 0"
               class="sample-data-preview">
            <h4>Sample Data ({{ datasetDetails.data.rows.length }} rows)</h4>
            <div class="table-container">
              <table class="preview-table">
                <thead>
                <tr>
                  <th v-for="header in datasetDetails.data.headers" :key="header">{{ header }}</th>
                </tr>
                </thead>
                <tbody>
                <tr v-for="(row, idx) in datasetDetails.data.rows.slice(0, 5)" :key="idx">
                  <td v-for="(cell, cellIdx) in row" :key="cellIdx">{{ cell }}</td>
                </tr>
                </tbody>
              </table>
              <p v-if="datasetDetails.data.rows.length > 5" class="text-muted">
                Showing 5 of {{ datasetDetails.data.rows.length }} rows
              </p>
            </div>
          </div>
        </div>

        <div class="modal-actions">
          <button class="btn btn-secondary" @click="showViewModal = false">
            Close
          </button>
        </div>
      </div>
    </div>

    <!-- Dataset Form (Create/Edit) -->
    <DatasetForm
        v-if="showDatasetForm"
        :dataset="editingDataset"
        :mode="datasetFormMode"
        @close="closeDatasetForm"
        @saved="handleDatasetSaved"
        @error="handleFormError"
    />
  </div>
</template>

<script setup lang="ts">
import {ref, onMounted} from 'vue'
import axios, {AxiosError} from 'axios'
import type {Dataset, GenerateFromDatasetRequest, GenerateFromDatasetResponse, ErrorResponse, CsvData} from '@/types'
import DatasetForm from './DatasetForm.vue'

const emit = defineEmits<{
  'data-generated': [data: GenerateFromDatasetResponse]
  'generation-error': [error: string]
}>()

const datasets = ref<Dataset[]>([])
const loading = ref<boolean>(true)
const error = ref<string | null>(null)

const showGenerateModal = ref<boolean>(false)
const showDeleteModal = ref<boolean>(false)
const showViewModal = ref<boolean>(false)
const showDatasetForm = ref<boolean>(false)
const selectedDataset = ref<Dataset | null>(null)
const editingDataset = ref<Dataset | undefined>(undefined)
const datasetFormMode = ref<'create' | 'edit'>('create')
const rowCount = ref<number>(20)
const generating = ref<boolean>(false)
const deleting = ref<boolean>(false)
const loadingDetails = ref<boolean>(false)
const datasetDetails = ref<{ dataset: Dataset; data: CsvData } | null>(null)

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

const openCreateForm = () => {
  datasetFormMode.value = 'create'
  editingDataset.value = undefined
  showDatasetForm.value = true
}

const openEditForm = (dataset: Dataset) => {
  datasetFormMode.value = 'edit'
  editingDataset.value = dataset
  showDatasetForm.value = true
}

const closeDatasetForm = () => {
  showDatasetForm.value = false
  editingDataset.value = undefined
}

const handleDatasetSaved = async () => {
  await fetchDatasets()
}

const handleFormError = (errorMessage: string) => {
  error.value = errorMessage
  setTimeout(() => {
    error.value = null
  }, 5000)
}

const openViewModal = async (dataset: Dataset) => {
  selectedDataset.value = dataset
  showViewModal.value = true
  loadingDetails.value = true
  datasetDetails.value = null

  try {
    const response = await axios.get<{ dataset: Dataset; data: CsvData }>(
        `/api/datasets/${dataset.id}`
    )
    datasetDetails.value = response.data
  } catch (err) {
    const axiosError = err as AxiosError<ErrorResponse>
    error.value = axiosError.response?.data?.error || 'Failed to load dataset details'
    showViewModal.value = false
  } finally {
    loadingDetails.value = false
  }
}

const duplicateDataset = async (dataset: Dataset) => {
  try {
    const response = await axios.post<{ id: number; message: string }>(
        `/api/datasets/${dataset.id}/duplicate`,
        {}
    )

    // Refresh the dataset list to show the new copy
    await fetchDatasets()

    // Show success message temporarily
    const successMessage = response.data.message
    // You could emit a success event here if needed
    console.log(successMessage)
  } catch (err) {
    const axiosError = err as AxiosError<ErrorResponse>
    error.value = axiosError.response?.data?.error || 'Failed to duplicate dataset'
    setTimeout(() => {
      error.value = null
    }, 5000)
  }
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
  gap: 0.3rem;
  margin: 0.5rem;
  flex-wrap: wrap;
  align-items: center;
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
  background: lightgray;
  padding: 1rem;
  margin: -2rem -2rem 0 -2rem;
  border-radius: 0 0 0.5rem 0.5rem;
}

.modal-large {
  max-width: 900px;
  background: white;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 1.5rem;
  background: white;
}

.modal-header h3 {
  margin: 0;
}

.btn-close {
  background: none;
  border: none;
  font-size: 2rem;
  line-height: 1;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-close:hover {
  color: var(--text-color);
}

.dataset-details {
  padding: 1rem 0;
  background: white;
}

.details-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
  margin-bottom: 2rem;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.75rem;
  background: var(--bg-secondary);
  border-radius: 0.25rem;
}

.detail-item .label {
  font-weight: 500;
  color: var(--text-muted);
}

.detail-item .value {
  font-weight: 600;
}

.columns-preview {
  margin-bottom: 2rem;
}

.columns-preview h4 {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 1rem;
}

.columns-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.column-tag {
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 0.25rem;
  font-size: 0.875rem;
  font-family: monospace;
}

.sample-data-preview h4 {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 1rem;
}

.table-container {
  overflow-x: auto;
  border: 1px solid var(--border-color);
  border-radius: 0.5rem;
}

.preview-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

.preview-table th,
.preview-table td {
  padding: 0.75rem;
  text-align: left;
  border-bottom: 1px solid var(--border-color);
}

.preview-table th {
  background: var(--bg-secondary);
  font-weight: 600;
  white-space: nowrap;
}

.preview-table tbody tr:last-child td {
  border-bottom: none;
}

.preview-table tbody tr:hover {
  background: var(--bg-secondary);
}
</style>
