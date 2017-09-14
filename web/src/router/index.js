import Vue from 'vue'
import Router from 'vue-router'
import Main from '@/components/Main'
import Upload from '@/components/Upload'
import Download from '@/components/Download'

Vue.use(Router)

export default new Router({
  routes: [
    {
      path: '/',
      name: 'Main',
      component: Main
    },
    {
      path: '/upload',
      name: 'Upload',
      component: Upload
    },
    {
      path: '/download',
      name: 'Download',
      component: Download
    }
  ]
})
