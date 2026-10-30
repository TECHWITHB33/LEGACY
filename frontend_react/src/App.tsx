import { Routes, Route, Navigate } from 'react-router-dom'
import { useEffect, useState } from 'react'

// Placeholder pages
const Dashboard = () => (
  <div className="p-8">
    <h1 className="text-3xl font-bold">Legacy Student Identity System</h1>
    <p className="text-gray-600 mt-2">Invoice Liquidity & Soroban Smart Contracts</p>
    <div className="mt-8 grid grid-cols-2 gap-4">
      <div className="p-4 border rounded">
        <h2 className="font-bold">Your Invoices</h2>
        <p className="text-sm text-gray-500">Manage your tokenized invoices</p>
      </div>
      <div className="p-4 border rounded">
        <h2 className="font-bold">Settlements</h2>
        <p className="text-sm text-gray-500">Track global advancement status</p>
      </div>
    </div>
  </div>
)

const Login = () => (
  <div className="flex items-center justify-center h-screen">
    <div className="w-96 p-8 border rounded shadow">
      <h1 className="text-2xl font-bold mb-4">Login</h1>
      <form className="space-y-4">
        <input
          type="email"
          placeholder="Email"
          className="w-full p-2 border rounded"
        />
        <input
          type="password"
          placeholder="Password"
          className="w-full p-2 border rounded"
        />
        <button className="w-full bg-blue-500 text-white p-2 rounded">
          Sign In
        </button>
      </form>
    </div>
  </div>
)

export default function App() {
  const [isAuthenticated, setIsAuthenticated] = useState(false)

  useEffect(() => {
    // Check if user is logged in
    const token = localStorage.getItem('auth_token')
    setIsAuthenticated(!!token)
  }, [])

  return (
    <div className="min-h-screen bg-gray-50">
      <nav className="bg-white border-b p-4">
        <div className="max-w-7xl mx-auto flex justify-between items-center">
          <h1 className="text-xl font-bold">Legacy</h1>
          {isAuthenticated && (
            <button
              onClick={() => {
                localStorage.removeItem('auth_token')
                setIsAuthenticated(false)
              }}
              className="text-sm text-gray-600 hover:text-gray-900"
            >
              Logout
            </button>
          )}
        </div>
      </nav>

      <Routes>
        {isAuthenticated ? (
          <>
            <Route path="/" element={<Dashboard />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </>
        ) : (
          <>
            <Route path="/login" element={<Login />} />
            <Route path="*" element={<Navigate to="/login" replace />} />
          </>
        )}
      </Routes>
    </div>
  )
}
