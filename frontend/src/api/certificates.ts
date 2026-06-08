import { api } from './client'
import type { CertificateInfo } from './types'

export function listCertificates() {
  return api<CertificateInfo[]>('/api/certificates')
}

export function createCertificate(name: string, certificatePem: string, privateKeyPem: string) {
  return api<CertificateInfo>('/api/certificates', {
    method: 'POST',
    json: {
      name,
      certificate_pem: certificatePem,
      private_key_pem: privateKeyPem,
    },
  })
}

export function deleteCertificate(id: string) {
  return api<{ ok: boolean }>(`/api/certificates/${id}`, { method: 'DELETE' })
}
