export class ApiError extends Error {
  readonly status: number

  constructor(message: string, status: number) {
    super(message)
    this.status = status
  }
}

interface RequestOptions extends RequestInit {
  json?: unknown
}

export async function api<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const headers = new Headers(options.headers)

  let body = options.body
  if (options.json !== undefined) {
    headers.set('Content-Type', 'application/json')
    body = JSON.stringify(options.json)
  }

  const response = await fetch(path, {
    ...options,
    headers,
    body,
    credentials: 'same-origin',
  })

  if (!response.ok) {
    let message = response.statusText
    try {
      const data = (await response.json()) as { error?: string }
      if (data.error) {
        message = data.error
      }
    } catch {
      const text = await response.text()
      if (text) {
        message = text
      }
    }
    throw new ApiError(message, response.status)
  }

  if (response.status === 204) {
    return undefined as T
  }

  return (await response.json()) as T
}
