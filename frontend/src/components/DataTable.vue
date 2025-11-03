<template>
  <div class="card">
    <div class="stats">
      <div class="stat-card">
        <div class="stat-value">{{ data.headers.length }}</div>
        <div class="stat-label">Columns</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ data.rows.length }}</div>
        <div class="stat-label">Rows</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ data.headers.length * data.rows.length }}</div>
        <div class="stat-label">Total Cells</div>
      </div>
    </div>

    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>#</th>
            <th v-for="(header, index) in data.headers" :key="index">
              {{ header }}
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, rowIndex) in displayedRows" :key="rowIndex">
            <td>{{ rowIndex + 1 }}</td>
            <td v-for="(cell, cellIndex) in row" :key="cellIndex">
              {{ cell }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="data.rows.length > maxDisplayRows" style="margin-top: 1rem; text-align: center;">
      <button
        v-if="!showAll"
        @click="showAll = true"
        class="btn btn-secondary"
      >
        Show all {{ data.rows.length }} rows
      </button>
      <button
        v-else
        @click="showAll = false"
        class="btn btn-secondary"
      >
        Show fewer rows
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { CsvData } from '@/types'

interface Props {
  data: CsvData
}

const props = defineProps<Props>()

const maxDisplayRows: number = 50
const showAll = ref<boolean>(false)

const displayedRows = computed<string[][]>(() => {
  if (showAll.value || props.data.rows.length <= maxDisplayRows) {
    return props.data.rows
  }
  return props.data.rows.slice(0, maxDisplayRows)
})
</script>
