<template>
  <div class="upload">
    <form id="upload-form" v-on:submit="upload">
      <div class="table-">
        <div class="row-">
          <div class="cell- left">
            Select a file:
          </div>
          <div class="cell- right">
            <input id="file" type="file"/>
          </div>
        </div>
        <div class="row-">
          <div class="cell- left">
            Access Password:
          </div>
          <div class="cell- right">
            <input id="access-password" v-model="accessPass" type="password"/>
          </div>
        </div>
        <div class="row-">
          <div class="cell- left">
            Encryption Password:
          </div>
          <div class="cell- right">
            <input id="encrypt-password" v-model="encryptPass" type="password"/>
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

export default {
  name: 'upload',
  data () {
    return {
      message: '',
      iv: '',
      accessPass: '',
      encryptPass: ''
    }
  },

  created () {
    console.log('hey')
  },

  methods: {
    upload () {
      let file = document.getElementById('file').files[0]
      if (!file) {
        console.log('file required')
        return
      }
      let iv = randomBytes(16)
      let encryptPassBytes = new TextEncoder().encode(this.encryptPass)
      let accessPassBytes = new TextEncoder().encode(this.accessPass)
      let ivHex = Buffer.from(iv).toString('hex')
      let accessPassHex = Buffer.from(accessPassBytes).toString('hex')

      const encryptUploadData = (data, key, respUrl) => {
        console.log(`upload key: ${key}`)
        console.log('freshbytes', data)
        const uploadBytesCallback = (bytes) => {
          console.log(bytes)
          let bytesHex = Buffer.from(bytes).toString('hex')
          axios.post(`${respUrl}?key=${key}`, bytesHex, {headers: {'content-type': 'text/plain'}})
            .then(resp => {
              console.log(resp.data)
              console.log(`key: ${key}`)
            })
        }
        encrypt(data, iv, encryptPassBytes, uploadBytesCallback)
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
          const params = {file_name: file.name, content_hash: contentHashHex, iv: ivHex, access_password: accessPassHex}
          const headers = {headers: {'content-type': 'application/json'}}
          axios.post('/api/upload/init', params, headers).then(resp => {
            encryptUploadData(data, resp.data.key, resp.data.responseUrl)
          }).catch(err => logerr(err))
        }).catch(err => logerr(err))
      }
      reader.readAsArrayBuffer(file)
    }
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
.table- {
  display: table;
}
.row- {
  display: table-row;
}
.cell- {
  display: table-cell;
}
.left {
  text-align: left;
}
.right {
  text-align: right;
}
</style>
