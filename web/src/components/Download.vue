<template>
  <div id="upload">
    <h3>
      Download
    </h3>
    <router-link to="/upload">upload</router-link>
    <form id="download-form" v-on:submit="download">
      <div class="table-">
        <div class="row-">
          <div class="cell- left">
            Download key:
          </div>
          <div class="cell- left">
            <input id="key" v-model="key" type="text"/>
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
          <button id="submit-form" type="submit" for="download-form">Download</button>
        </div>
      </div>
    </form>

    <div id="error-message">
      <span v-if="errorMessage">{{ errorMessage }}</span>
    </div>
  </div>
</template>

<script>
import axios from 'axios'
import FileSaver from 'file-saver'
import Password from '@/components/Password'
import {logerr} from '@/utils/error'
import {bytesFromHex, decrypt} from '@/utils/crypto'

export default {
  name: 'download',
  components: {
    Password
  },
  data () {
    return {
      key: '',
      accessPass: '',
      encryptPass: '',
      errorMessage: ''
    }
  },

  created () {
    let params = this.$route.query
    console.log(params)
    this.key = params.key
  },

  methods: {
    updateField (field, val) {
      this[field] = val
    },
    download () {
      this.errorMessage = ''
      if (this.key.length === 0) {
        console.log('key required')
        return
      }
      if (this.accessPass.length === 0) {
        console.log('access pass required')
        return
      }
      if (this.encryptPass.length === 0) {
        console.log('encypt pass required')
        return
      }
      const decryptedBytesCallback = (bytes) => {
        window.crypto.subtle.digest('SHA-256', bytes).then(contentHash => {
          const hex = Buffer.from(contentHash).toString('hex')
          console.log('decrypted bytes hex', hex)
          const params = {key: this.key, hash: hex}
          const headers = {headers: {'content-type': 'application/json'}}
          axios.post('/api/download/name', params, headers).then(resp => {
            const blob = new Blob([bytes], {type: 'application/octet-stream'})
            FileSaver.saveAs(blob, resp.data.file_name)
          }).catch(logerr)
        }).catch(logerr)
      }

      const decryptionFailedCallback = e => {
        logerr(e)
        this.errorMessage = 'Decryption Failed'
      }

      const params = {key: this.key, access_password: Buffer.from(this.accessPass).toString('hex')}
      const headers = {headers: {'content-type': 'application/json'}}
      axios.post('/api/download/init', params, headers).then(resp => {
        const nonce = new Uint8Array(bytesFromHex(resp.data.nonce))
        console.log(`nonce: ${nonce}`)
        const DLheaders = {headers: {'content-type': 'application/json'}, responseType: 'text'}
        axios.post('/api/download', params, DLheaders).then(resp => {
          console.log('post dl')
          console.log(resp)
          const dataBytes = new Uint8Array(bytesFromHex(resp.data))
          console.log(dataBytes)
          const encryptPassBytes = new TextEncoder().encode(this.encryptPass)
          console.log('encrytion pass', encryptPassBytes)
          decrypt(dataBytes, nonce, encryptPassBytes, decryptedBytesCallback, decryptionFailedCallback)
        }).catch(logerr)
      }).catch(logerr)
    }
  }
}

</script>

<style scoped>
#error-message {
  color: red;
}
</style>
