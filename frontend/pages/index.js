import axios from 'axios'
import { useState } from 'react'

export default function Home() {
  const [query, setQuery] = useState('')
  const [result, setResult] = useState(null)
  const [loading, setLoading] = useState(false)

  const submit = async () => {
    setLoading(true)
    try {
      const res = await axios.get(`/v1/scan?query=${encodeURIComponent(query)}`, {
        headers: { 'X-Spore-Signature': process.env.NEXT_PUBLIC_SPORE_SIGNATURE }
      })
      setResult(res.data)
    } catch (e) {
      setResult({ error: e.toString() })
    }
    setLoading(false)
  }

  return (
    <div className="min-h-screen bg-gray-100 p-8">
      <div className="max-w-3xl mx-auto bg-white shadow p-6 rounded">
        <h1 className="text-2xl font-bold mb-4">SPORESEC Dashboard (Local)</h1>
        <div className="mb-4">
          <input value={query} onChange={(e) => setQuery(e.target.value)} className="w-full p-2 border rounded" placeholder="Search query" />
        </div>
        <div className="flex gap-2">
          <button onClick={submit} disabled={loading} className="px-4 py-2 bg-blue-600 text-white rounded">{loading ? 'Scanning...' : 'Scan'}</button>
        </div>
        <pre className="mt-4 text-sm bg-gray-50 p-4 rounded">{result ? JSON.stringify(result, null, 2) : 'No results yet'}</pre>
      </div>
    </div>
  )
}
