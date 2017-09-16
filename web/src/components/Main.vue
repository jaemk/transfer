<template>
  <div class="main">
    <h1>{{ message }}</h1>
    <button v-on:click="bye">Say Bye</button>
    <br/>
    <router-link to="/upload">upload</router-link>
    <br/>
    <router-link to="/download">download</router-link>
  </div>
</template>

<script>

export default {
  name: 'main',
  data () {
    return {
      message: ''
    }
  },

  created () {
    console.log('hey')
  },

  methods: {
  }
}
</script>

<style>
</style>

