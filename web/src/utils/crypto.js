const crypto = window.crypto
const algoName = 'AES-CBC'

export const randomBytes = (n) => {
  return crypto.getRandomValues(new Uint8Array(n))
}

export const InvalidHexException = (s) => {
  this.message = s
  this.name = 'InvalidHexException'
}

export const bytesFromHex = (s) => {
  if (s.length % 2 !== 0) { throw new InvalidHexException('Invalid hex string') }
  const chars = Array.from(s)
  let buf = []
  for (let i = 0; i < chars.length - 2; i += 2) {
    const hexDigit = chars.slice(i, i + 2).join('')
    buf.push(parseInt(hexDigit, 16))
  }
  return buf
}

export const encrypt = (data, iv, pass, callback) => {
  const algo = {name: algoName, iv: iv}
  crypto.subtle.digest('SHA-256', pass).then((hash) => {
    // console.log(`hash: ${hash}`)
    crypto.subtle.importKey('raw', hash, algo, false, ['encrypt']).then((key) => {
      // console.log(`key: ${key}`)
      crypto.subtle.encrypt(algo, key, data).then((encryptedData) => {
        // console.log(encryptedData)
        let dataBytes = new Uint8Array(encryptedData)
        dataBytes = Array.from(dataBytes)
        callback(dataBytes)
      })
    })
  })
}

export const decrypt = (data, iv, pass, callback) => {
  const algo = {name: algoName, iv: iv}
  crypto.subtle.digest('SHA-256', pass).then((hash) => {
    crypto.subtle.importKey('raw', hash, algo, false, ['decrypt']).then(key => {
      console.log('got key')
      crypto.subtle.decrypt(algo, key, data).then(decryptedData => {
        console.log('decr')
        let dataBytes = new Uint8Array(decryptedData)
        dataBytes = Array.from(dataBytes)
        console.log(dataBytes)
        callback(dataBytes)
      }).catch(e => console.log(e))
    })
  })
}
