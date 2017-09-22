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

export const encrypt = (data, nonce, pass, callback) => {
  crypto.subtle.digest('SHA-256', pass).then(passHash => {
    const algo = {name: algoName, iv: nonce}
    console.log(nonce)
    crypto.subtle.importKey('raw', passHash, algo, false, ['encrypt']).then((key) => {
      crypto.subtle.encrypt(algo, key, data).then((encryptedData) => {
        let dataBytes = new Uint8Array(encryptedData)
        callback(dataBytes)
      }).catch(e => console.log('encrypt err:', e))
    }).catch(e => console.log('key err:', e))
  }).catch(logerr)
}

export const decrypt = (data, nonce, pass, decryptedCallback, decryptionFailedCallback) => {
  crypto.subtle.digest('SHA-256', pass).then(passHash => {
    const algo = {name: algoName, iv: nonce}
    crypto.subtle.importKey('raw', passHash, algo, false, ['decrypt']).then(key => {
      crypto.subtle.decrypt(algo, key, data).then(decryptedData => {
        let dataBytes = new Uint8Array(decryptedData)
        decryptedCallback(dataBytes)
      }).catch(e => decryptionFailedCallback ? decryptionFailedCallback(e) : logerr(e))
    }).catch(logerr)
  }).catch(logerr)
}
