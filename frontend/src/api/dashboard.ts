import { api } from './client'
import type { DashboardSummary } from './types'

export function getDashboardSummary() {
  return api<DashboardSummary>('/api/dashboard/summary')
}
