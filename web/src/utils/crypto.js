import {logerr, InvalidHexException} from './errors'


export const crypto = window.crypto
const algoName = 'AES-GCM'


/**
 * randomBytes
 *
 * @param n {int}
 * @returns {Uint8Array}
 */
export const randomBytes = (n) => {
  return crypto.getRandomValues(new Uint8Array(n))
}


/**
 * bytesFromHex
 *
 * @param s - {string}
 * @returns {Array}
 */
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


/**
 * encrypt
 * - encrypt AES-GCM-256
 *
 * @param data - `BufferSource`
 * @param nonce - `BufferSource`
 * @param pass - password text as `Uint8Array`
 * @param callback - function to receive encrypted bytes
 * @returns {undefined}
 */
export const encrypt = (data, nonce, pass, callback) => {
  crypto.subtle.digest('SHA-256', pass).then(passHash => {
    const algo = {name: algoName, iv: nonce}
    // console.log(nonce)
    crypto.subtle.importKey('raw', passHash, algo, false, ['encrypt']).then((key) => {
      crypto.subtle.encrypt(algo, key, data).then((encryptedData) => {
        let dataBytes = new Uint8Array(encryptedData)
        callback(dataBytes)
      }).catch(e => console.log('encrypt err:', e))
    }).catch(e => console.log('key err:', e))
  }).catch(logerr)
}


/**
 * decrypt
 *
 * - decrypt AES-GCM-256
 *
 * @param data - `BufferSource`
 * @param nonce - `BufferSource`
 * @param pass - password text as `Uint8Array`
 * @param decryptedCallback - function to receive encrypted bytes
 * @param decryptionFailedCallback - function to be called when decryption fails
 * @returns {undefined}
 */
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

