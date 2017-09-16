<template>
  <div class="tr-pass">
    <input v-model="pass" type="password" placeholder="password"/>
    <input v-if="confirm" v-model="passConfirm" type="password" placeholder="confirm"/>
    <span v-if="confirm"> {{ message }} </span>
  </div>
</template>

<script>
import _ from 'lodash'

export default {
  name: 'password',
  props: [
    'confirm',
    'updateFunc'
  ],
  data () {
    return {
      pass: '',
      passConfirm: '',
      message: ''
    }
  },

  watch: {
    pass () { this.compare() },
    passConfirm () { this.compare() }
  },

  methods: {
    compare () {
      _.debounce(
        () => {
          if (this.pass.length > 0 || this.passConfirm.length > 0) {
            if (this.pass === this.passConfirm) {
              this.message = 'check!'
              this.updateFunc(this.pass)
            } else {
              this.message = 'passwords do not match!'
              this.updateFunc('')
            }
          }
        },
        500
      )()
    }
  }

}
</script>

<style scoped>
</style>
