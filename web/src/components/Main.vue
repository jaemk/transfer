<template>
  <div class="main">
    <h1>{{ message }}</h1>
    <button v-on:click="bye">Say Bye</button>
    <br/>
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

export default {
  name: 'main',
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
    axios.get('/api/hello')
    .then(resp => {
      console.log(resp.data)
      this.message = resp.data.message
    })
    .catch(e => console.log(e))
  },

  methods: {
    bye () {
      console.log('bye')
      axios.post('/api/bye', {message: 'bye'}, {headers: {'content-type': 'application/json'}})
        .then(resp => {
          console.log(resp.data)
          this.message = resp.data.message
        })
        .catch(e => console.log(e))
    },
    upload () {
      let file = document.getElementById('file').files[0]
      if (!file) {
        console.log('file required')
        return
      }
      let iv = randomBytes(16)
      let encryptBytes = new TextEncoder().encode(this.encryptPass)
      let accessBytes = new TextEncoder().encode(this.accessPass)
      let ivHex = Buffer.from(iv).toString('hex')
      let accessHex = Buffer.from(accessBytes).toString('hex')
      let encryptHex = Buffer.from(encryptBytes).toString('hex')
      console.log(`Hex: ${ivHex} - ${accessHex} - ${encryptHex}`)

      const encryptUploadFile = (uuid) => {
        console.log(`uuid: ${uuid}`)
        const uploadBytes = (bytes, hash) => {
          console.log(bytes)
          axios.post(`/api/upload?uuid=${uuid}&hash=${hash}`, Buffer.from(bytes).toString('hex'), {headers: {'content-type': 'text/plain'}})
            .then(resp => console.log(resp.data))
        }
        encrypt(file, iv, encryptBytes, uploadBytes)
      }

      axios.post('/api/upload/init',
        {
          filename: file.name,
          iv: ivHex,
          access_password: accessHex,
          encrypt_password: encryptHex
        },
        {headers: {'content-type': 'application/json'}}
      ).then(resp => {
        let uuid = resp.data.uuid
        encryptUploadFile(uuid)
      }).catch(e => console.log(e))
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
