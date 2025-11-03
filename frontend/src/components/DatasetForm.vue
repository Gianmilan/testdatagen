<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal modal-large">
      <div class="modal-header">
        <h3>{{ isEditMode ? 'Edit Dataset' : 'Create Custom Dataset' }}</h3>
        <button class="btn-close" @click="$emit('close')">&times;</button>
      </div>

      <div class="modal-body">
        <!-- Dataset Name -->
        <div class="form-group">
          <label for="dataset-name">Dataset Name *</label>
          <input
              id="dataset-name"
              v-model="formData.name"
              type="text"
              class="input"
              placeholder="e.g., Customer Data, Product Catalog"
              :class="{ 'input-error': errors.name }"
              @input="validateName"
          />
          <span v-if="errors.name" class="error-message">{{ errors.name }}</span>
        </div>

        <!-- Columns Section -->
        <div class="columns-section">
          <div class="section-header">
            <h4>Columns</h4>
            <button
                class="btn btn-secondary btn-sm"
                @click="addColumn"
                :disabled="formData.columns.length >= 50"
            >
              + Add Column
            </button>
          </div>

          <div v-if="formData.columns.length === 0" class="empty-columns">
            <p>No columns yet. Click "Add Column" to get started.</p>
          </div>

          <div v-else class="columns-list">
            <div
                v-for="(column, index) in formData.columns"
                :key="column.id"
                class="column-item"
            >
              <div class="column-number">{{ index + 1 }}</div>

              <div class="column-field">
                <label :for="`col-name-${column.id}`">Column Name</label>
                <input
                    :id="`col-name-${column.id}`"
                    v-model="column.name"
                    type="text"
                    class="input"
                    placeholder="e.g., customer_id, email, age"
                    :class="{ 'input-error': errors.columns[column.id] }"
                    @input="validateColumn(column.id)"
                />
                <span v-if="errors.columns[column.id]" class="error-message">
                  {{ errors.columns[column.id] }}
                </span>
              </div>

              <div class="column-field">
                <label :for="`col-type-${column.id}`">Data Type</label>
                <select
                    :id="`col-type-${column.id}`"
                    v-model="column.type"
                    class="input"
                >
                  <option
                      v-for="option in COLUMN_TYPES"
                      :key="option.value"
                      :value="option.value"
                  >
                    {{ option.label }}
                  </option>
                </select>
              </div>

              <button
                  class="btn btn-icon btn-danger"
                  @click="removeColumn(column.id)"
                  :disabled="formData.columns.length === 1"
                  title="Remove column"
              >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"
                     stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18"></line>
                  <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
              </button>
            </div>
          </div>

          <p v-if="formData.columns.length > 0" class="column-count-info">
            {{ formData.columns.length }} column{{ formData.columns.length !== 1 ? 's' : '' }}
            (max 50)
          </p>
        </div>

        <!-- Info Box -->
        <div class="info-box">
          <strong>Note:</strong> This creates a schema-only dataset. You can generate test data
          from this schema later using the FlexibleGenerator with the types you've specified.
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" @click="$emit('close')">
          Cancel
        </button>
        <button
            class="btn btn-primary"
            @click="handleSubmit"
            :disabled="!isFormValid || saving"
        >
          {{ saving ? 'Saving...' : (isEditMode ? 'Update Dataset' : 'Create Dataset') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {ref, computed, onMounted} from 'vue'
import axios, {AxiosError} from 'axios'
import type {SaveDatasetRequest, SaveDatasetResponse, ErrorResponse, Dataset, CsvData} from '@/types'

// Column type options based on FlexibleGenerator
const COLUMN_TYPES = [
  {value: 'auto', label: 'Auto Detect'},
  {value: 'id', label: 'ID'},
  {value: 'name', label: 'Name'},
  {value: 'email', label: 'Email'},
  {value: 'age', label: 'Age'},
  {value: 'city', label: 'City'},
  {value: 'country', label: 'Country'},
  {value: 'phone', label: 'Phone'},
  {value: 'date', label: 'Date'},
  {value: 'money', label: 'Money'},
  {value: 'text', label: 'Text'}
]

interface Column {
  id: string
  name: string
  type: string
}

interface FormData {
  name: string
  columns: Column[]
}

interface Errors {
  name: string | null
  columns: Record<string, string>
}

// Props
const props = defineProps<{
  dataset?: Dataset
  mode?: 'create' | 'edit'
}>()

// Emits
const emit = defineEmits<{
  'close': []
  'saved': [datasetId: number]
  'error': [error: string]
}>()

// State
const formData = ref<FormData>({
  name: '',
  columns: []
})

const errors = ref<Errors>({
  name: null,
  columns: {}
})

const saving = ref(false)
let nextColumnId = 0

// Computed
const isEditMode = computed(() => props.mode === 'edit' && props.dataset)

const isFormValid = computed(() => {
  return (
      formData.value.name.trim().length > 0 &&
      formData.value.columns.length > 0 &&
      formData.value.columns.every(col => col.name.trim().length > 0) &&
      !errors.value.name &&
      Object.keys(errors.value.columns).length === 0
  )
})

// Initialize form
onMounted(async () => {
  if (isEditMode.value && props.dataset) {
    // Load dataset data for editing
    formData.value.name = props.dataset.name

    try {
      // Fetch full dataset details including headers and column types
      const response = await axios.get<{ dataset: Dataset; data: CsvData }>(
        `/api/datasets/${props.dataset.id}`
      )

      const { dataset, data } = response.data

      // Parse column_types if they exist (stored as JSON string in database)
      let columnTypesMap: Record<string, string> = {}
      if (dataset.column_types) {
        try {
          columnTypesMap = JSON.parse(dataset.column_types)
        } catch (e) {
          console.warn('Failed to parse column_types:', e)
        }
      }

      // Load columns from dataset headers
      data.headers.forEach(headerName => {
        const id = `col-${nextColumnId++}`
        formData.value.columns.push({
          id,
          name: headerName,
          type: columnTypesMap[headerName] || 'auto'
        })
      })
    } catch (err) {
      const axiosError = err as AxiosError<ErrorResponse>
      const errorMessage = axiosError.response?.data?.error || 'Failed to load dataset details'
      emit('error', errorMessage)
      // Add at least one empty column as fallback
      addColumn()
    }
  } else {
    // Start with one empty column for new datasets
    addColumn()
  }
})

// Methods
const addColumn = () => {
  const id = `col-${nextColumnId++}`
  formData.value.columns.push({
    id,
    name: '',
    type: 'auto'
  })
}

const removeColumn = (id: string) => {
  formData.value.columns = formData.value.columns.filter(col => col.id !== id)
  delete errors.value.columns[id]
}

const validateName = () => {
  const name = formData.value.name.trim()

  if (name.length === 0) {
    errors.value.name = 'Dataset name is required'
  } else if (name.length > 255) {
    errors.value.name = 'Dataset name must be 255 characters or less'
  } else {
    errors.value.name = null
  }
}

const validateColumn = (id: string) => {
  const column = formData.value.columns.find(col => col.id === id)
  if (!column) return

  const name = column.name.trim()

  if (name.length === 0) {
    errors.value.columns[id] = 'Column name is required'
  } else {
    // Check for duplicate names
    const duplicates = formData.value.columns.filter(col =>
        col.name.trim().toLowerCase() === name.toLowerCase() && col.id !== id
    )

    if (duplicates.length > 0) {
      errors.value.columns[id] = 'Duplicate column name'
    } else {
      delete errors.value.columns[id]
    }
  }
}

const handleSubmit = async () => {
  // Validate all fields
  validateName()
  formData.value.columns.forEach(col => validateColumn(col.id))

  if (!isFormValid.value) {
    return
  }

  saving.value = true

  try {
    const headers = formData.value.columns.map(col => col.name.trim())

    // Build column types map (exclude 'auto' types)
    const columnTypes: Record<string, string> = {}
    formData.value.columns.forEach(col => {
      if (col.type !== 'auto') {
        columnTypes[col.name.trim()] = col.type
      }
    })

    const request: SaveDatasetRequest = {
      name: formData.value.name.trim(),
      headers,
      data_type: 'custom',
      column_types: Object.keys(columnTypes).length > 0 ? columnTypes : undefined,
      sample_data: undefined
    }

    if (isEditMode.value && props.dataset) {
      // Update existing dataset
      const response = await axios.put<SaveDatasetResponse>(
          `/api/datasets/${props.dataset.id}`,
          request
      )
      emit('saved', response.data.id)
    } else {
      const response = await axios.post<SaveDatasetResponse>(
          '/api/datasets',
          request
      )
      emit('saved', response.data.id)
    }

    emit('close')
  } catch (err) {
    const axiosError = err as AxiosError<ErrorResponse>
    const errorMessage = axiosError.response?.data?.error || 'Failed to save dataset'
    emit('error', errorMessage)
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.modal-large {
  max-width: 800px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-bottom: 1rem;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 1.5rem;
}

.modal-header h3 {
  margin: 0;
  font-size: 1.25rem;
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

.modal-body {
  flex: 1;
  overflow-y: auto;
  padding-right: 0.5rem;
}

.modal-footer {
  display: flex;
  gap: 0.5rem;
  justify-content: flex-end;
  padding-top: 1.5rem;
  border-top: 1px solid var(--border-color);
  margin-top: 1.5rem;
}

.form-group {
  margin-bottom: 1.5rem;
}

.form-group label {
  display: block;
  font-weight: 500;
  margin-bottom: 0.5rem;
  font-size: 0.875rem;
}

.columns-section {
  margin-bottom: 1.5rem;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1rem;
}

.section-header h4 {
  margin: 0;
  font-size: 1rem;
  font-weight: 600;
}

.empty-columns {
  text-align: center;
  padding: 2rem;
  background: var(--bg-secondary);
  border-radius: 0.5rem;
  color: var(--text-muted);
}

.columns-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.column-item {
  display: grid;
  grid-template-columns: 2rem 1fr 1fr 2.5rem;
  gap: 0.75rem;
  align-items: start;
  padding: 1rem;
  background: var(--bg-secondary);
  border-radius: 0.5rem;
  border: 1px solid var(--border-color);
}

.column-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 2rem;
  height: 2rem;
  background: var(--primary-color);
  color: white;
  border-radius: 50%;
  font-size: 0.75rem;
  font-weight: 600;
  margin-top: 1.5rem;
}

.column-field {
  display: flex;
  flex-direction: column;
}

.column-field label {
  font-size: 0.75rem;
  font-weight: 500;
  margin-bottom: 0.25rem;
  color: var(--text-muted);
}

.btn-icon {
  padding: 0.5rem;
  min-width: 2rem;
  height: 2.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 1.5rem;
}

.btn-icon svg {
  width: 1rem;
  height: 1rem;
}

.column-count-info {
  margin-top: 0.5rem;
  font-size: 0.875rem;
  color: var(--text-muted);
  text-align: right;
}

.info-box {
  padding: 1rem;
  background: #e0f2fe;
  border-left: 3px solid #0284c7;
  border-radius: 0.25rem;
  font-size: 0.875rem;
  line-height: 1.5;
}

.input-error {
  border-color: #ef4444;
}

.error-message {
  color: #ef4444;
  font-size: 0.75rem;
  margin-top: 0.25rem;
}
</style>
