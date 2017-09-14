<template>
  <div class="main">
    <h1>{{ message }}</h1>
    <button v-on:click="bye">Say Bye</button>
    <br/>
    <router-link to="/upload">upload</router-link>
    <router-link to="/download">download</router-link>
  </div>
</template>

<script>
import axios from 'axios'

export default {
  name: 'main',
  data () {
    return {
      message: ''
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
    }
  }
}
</script>
