import {logerr, InvalidHexException} from '@/utils/error'

export const crypto = window.crypto
const algoName = 'AES-GCM'

export const randomBytes = (n) => {
  return crypto.getRandomValues(new Uint8Array(n))
}

export const bytesFromHex = (s) => {
  if (s.length % 2 !== 0) { throw new InvalidHexException('Invalid hex string') }
  const chars = Array.from(s)
  let buf = []
  for (let i = 0; i <= chars.length - 2; i += 2) {
    const hexDigit = chars.slice(i, i + 2).join('')
    buf.push(parseInt(hexDigit, 16))
  }
  return buf
}

export const encrypt = (data, iv, pass, callback) => {
  const algo = {name: algoName, iv: iv}
  crypto.subtle.digest('SHA-256', pass).then((hash) => {
    crypto.subtle.importKey('raw', hash, algo, false, ['encrypt']).then((key) => {
      crypto.subtle.encrypt(algo, key, data).then((encryptedData) => {
        let dataBytes = new Uint8Array(encryptedData)
        callback(dataBytes)
      })
    })
  })
}

export const decrypt = (data, iv, pass, decryptedCallback, decryptionFailedCallback) => {
  const algo = {name: algoName, iv: iv}
  crypto.subtle.digest('SHA-256', pass).then((hash) => {
    crypto.subtle.importKey('raw', hash, algo, false, ['decrypt']).then(key => {
      crypto.subtle.decrypt(algo, key, data).then(decryptedData => {
        let dataBytes = new Uint8Array(decryptedData)
        decryptedCallback(dataBytes)
      }).catch(e => decryptionFailedCallback ? decryptionFailedCallback(e) : logerr(e))
    }).catch(logerr)
  }).catch(logerr)
}
