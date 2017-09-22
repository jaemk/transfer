<template>
  <div class="upload">
    <h3>
      Upload
    </h3>
    <a v-if="uploaded" :href="downloadUrl">{{downloadUrl}}</a>
    <router-link v-else to="/download">download</router-link>
    <form id="upload-form" v-on:submit="upload">
      <div class="table-">
        <div class="row-">
          <div class="cell- left">
            Select a file:
          </div>
          <div class="cell- left">
            <input id="file" type="file"/>
          </div>
        </div>
        <div class="row-">
          <div class="cell- left">
            Access Password:
          </div>
          <div class="cell- left">
            <Password confirm=true :updateFunc="(val) => updateField('accessPass', val)" />
          </div>
        </div>
        <div class="row-">
          <div class="cell- left">
            Encryption Password:
          </div>
          <div class="cell- left">
            <Password confirm=true :updateFunc="(val) => updateField('encryptPass', val)" />
          </div>
        </div>
        <div cell="row-">
          <button id="submit-form" type="submit" for="upload-form">Upload</button>
        </div>
      </div>
    </form>
  </div>
</template>

<script>
import axios from 'axios'
import {encrypt, randomBytes} from '@/utils/crypto'
import {logerr} from '@/utils/error'
import Password from '@/components/Password'

export default {
  name: 'upload',
  components: {
    Password
  },

  data () {
    return {
      accessPass: '',
      encryptPass: '',
      iv: '',
      uploaded: false,
      downloadUrl: ''
    }
  },

  methods: {
    updateField (field, val) {
      this[field] = val
    },
    upload () {
      let file = document.getElementById('file').files[0]
      if (!file) {
        console.log('file required')
        return
      }
      if (this.accessPass.length === 0) {
        console.log('access pass required')
        return
      }
      if (this.encryptPass.length === 0) {
        console.log('encrypt pass required')
        return
      }
      let nonce = randomBytes(12)
      let encryptPassBytes = new TextEncoder().encode(this.encryptPass)
      let accessPassBytes = new TextEncoder().encode(this.accessPass)
      let nonceHex = Buffer.from(nonce).toString('hex')
      let accessPassHex = Buffer.from(accessPassBytes).toString('hex')

      const encryptUploadData = (data, key, respUrl) => {
        console.log(`upload key: ${key}`)
        console.log('freshbytes', data)

        const encryptedBytesCallback = (bytes) => {
          console.log('encrypted bytes', bytes.length, bytes)
          let bytesHex = Buffer.from(bytes).toString('hex')
          axios.post(`${respUrl}?key=${key}`, bytesHex, {headers: {'content-type': 'text/plain'}})
            .then(resp => {
              console.log(resp.data)
              console.log(`key: ${key}`)
              this.uploaded = true
              this.downloadUrl = `http://${window.location.host}/#/download?key=${key}`
            }).catch(logerr)
        }
        encrypt(data, nonce, encryptPassBytes, encryptedBytesCallback)
      }

      let reader = new FileReader()
      reader.onload = (event) => {
        if (reader.readyState !== 2) {
          console.log(`read ${event.loaded} bytes`)
          return
        }
        const data = reader.result
        console.log(`loaded ${data.byteLength} bytes`)
        window.crypto.subtle.digest('SHA-256', data).then(contentHash => {
          const contentHashHex = Buffer.from(contentHash).toString('Hex')
          console.log('content hash', contentHashHex)
          const params = {file_name: file.name, file_size: data.byteLength, content_hash: contentHashHex, nonce: nonceHex, access_password: accessPassHex}
          const headers = {headers: {'content-type': 'application/json'}}
          axios.post('/api/upload/init', params, headers).then(resp => {
            encryptUploadData(data, resp.data.key, resp.data.response_url)
          }).catch(logerr)
        }).catch(logerr)
      }
      reader.readAsArrayBuffer(file)
    }
  }
}
</script>

<style scoped>
</style>
