<template>
  <div id="upload">
    sup : {{ key }}
    <br/>
    <form id="download-form" v-on:submit="download">
      <div class="table-">
        <div class="row-">
          <div class="cell- left">
            download key:
          </div>
          <div class="cell- right">
            <input id="key" v-model="key" type="text"/>
          </div>
        </div>
        <div class="row-">
          <div class="cell- left">
            Access Password:
          </div>
          <div class="cell- right">
            <input id="access-pass" v-model="accessPass" type="password"/>
          </div>
        </div>
        <div class="row-">
          <div class="cell- left">
            Encryption Password:
          </div>
          <div class="cell- right">
            <input id="encrypt-pass" v-model="encryptPass" type="password"/>
          </div>
        </div>
        <div cell="row-">
          <button id="submit-form" type="submit" for="download-form">Download</button>
        </div>
      </div>
    </form>
  </div>
</template>

<script>
import axios from 'axios'
import {logerr} from '@/utils/error'
import {bytesFromHex, decrypt} from '@/utils/crypto'

export default {
  name: 'download',
  data () {
    return {
      key: '',
      accessPass: '',
      encryptPass: ''
    }
  },

  created () {
    let params = this.$route.query
    console.log(params)
    this.key = params.key
  },

  methods: {
    download () {
      const params = {key: this.key, access_password: Buffer.from(this.accessPass).toString('hex')}
      const headers = {headers: {'content-type': 'application/json'}}
      axios.post('/api/download/iv', params, headers).then(resp => {
        const iv = bytesFromHex(resp.data.iv)
        console.log(`iv: ${iv}`)
        const DLheaders = {headers: {'content-type': 'application/json'}, responseType: 'text'}
        axios.post('/api/download', params, DLheaders).then(resp => {
          console.log('post dl')
          console.log(resp)
          const dataBytes = new Uint8Array(bytesFromHex(resp.data))
          console.log(dataBytes)
          const encryptPassBytes = new TextEncoder().encode(this.encryptPass)
          console.log('encrytion pass', encryptPassBytes)
          decrypt(dataBytes, iv, encryptPassBytes, (bytes) => console.log('decryptedbytes', bytes))
        })
      }).catch(err => logerr(err))
    }
  }
}

</script>

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
