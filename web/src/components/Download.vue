<template>
  <div id="upload">
    sup : {{ uuid }}
    <br/>
    <form id="download-form" v-on:submit="download">
      <input id="uuid" v-model="uuid" type="text"/>
      <input id="access-pass" v-model="accessPass" type="password"/>
      <input id="encrypt-pass" v-model="encryptPass" type="password"/>
      <button id="submit-form" type="submit" for="download-form">Upload</button>
    </form>
  </div>
</template>

<script>
export default {
  name: 'download',
  data () {
    return {
      uuid: '',
      accessPass: '',
      encryptPass: ''
    }
  },

  created () {
    let params = this.$route.query
    console.log(params)
    this.uuid = params.id
  },

  methods: {
    download () {
      console.log('download')
    }
  }
}

</script>
