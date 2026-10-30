import axios, { AxiosInstance, AxiosRequestConfig } from 'axios'

/**
 * API client for LEGACY backend
 * Uses VITE_API_URL environment variable when available
 * Falls back to /api for local development
 */

const getApiBaseUrl = (): string => {
  // In Vercel, VITE_API_URL is injected as a binding variable
  if (import.meta.env.VITE_API_URL) {
    return import.meta.env.VITE_API_URL
  }
  // Local development fallback
  if (import.meta.env.DEV) {
    return 'http://localhost:8080'
  }
  // Production fallback (relative to same domain)
  return '/api'
}

const client: AxiosInstance = axios.create({
  baseURL: getApiBaseUrl(),
  headers: {
    'Content-Type': 'application/json'
  }
})

/**
 * Interceptor to add JWT token to all requests
 */
client.interceptors.request.use((config) => {
  const token = localStorage.getItem('auth_token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

/**
 * Interceptor to handle 401 responses
 */
client.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('auth_token')
      window.location.href = '/login'
    }
    return Promise.reject(error)
  }
)

export default client

// API methods
export const api = {
  // Auth
  login: (email: string, password: string) =>
    client.post('/auth/login', { email, password }),
  register: (data: any) =>
    client.post('/auth/register', data),
  
  // Invoices
  createInvoice: (data: any) =>
    client.post('/escrow/invoices', data),
  getInvoice: (id: string) =>
    client.get(`/escrow/invoices/${id}`),
  listInvoices: (page = 1, limit = 10) =>
    client.get('/escrow/invoices', { params: { page, limit } }),
  submitInvoice: (id: string) =>
    client.put(`/escrow/invoices/${id}/submit`, {}),
  fundInvoice: (id: string, data: any) =>
    client.put(`/escrow/invoices/${id}/fund`, data),
  settleInvoice: (id: string, data: any) =>
    client.put(`/escrow/invoices/${id}/settle`, data),
  
  // Escrow
  getEscrowStatus: (id: string) =>
    client.get(`/escrow/escrows/${id}/status`)
}
