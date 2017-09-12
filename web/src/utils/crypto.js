const crypt = window.crypto

export const randomBytes = (n) => {
  return crypt.getRandomValues(new Uint8Array(n))
}

export const encrypt = (file, iv, pass, callback) => {
  const algo = {name: 'AES-CBC', iv: iv}
  let reader = new FileReader()
  reader.onload = (event) => {
    let data = event.target.result
    crypt.subtle.digest('SHA-256', pass).then((hash) => {
      // console.log(`hash: ${hash}`)
      crypt.subtle.importKey('raw', hash, algo, false, ['encrypt']).then((key) => {
        // console.log(`key: ${key}`)
        crypt.subtle.encrypt(algo, key, data).then((encryptedData) => {
          // console.log(encryptedData)
          let dataBytes = new Uint8Array(encryptedData)
          dataBytes = Array.from(dataBytes)

          crypt.subtle.digest('SHA-256', data).then(dataHash => {
            callback(dataBytes, Buffer.from(hash).toString('hex'))
          })
        })
      })
    })
  }
  reader.readAsArrayBuffer(file)
}
