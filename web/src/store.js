import Vuex from 'vuex'
import Vue from 'vue'

Vue.use(Vuex)

export default new Vuex.Store({
  state: {
    count: 0
  },

  mutations: {
    inc (state) {
      state.count++
    },
    incBy (state, { val }) {
      state.count += val
    }
  },

  actions: {
    inc ({commit}) {
      commit('inc')
    },
    incBy ({commit}, value) {
      commit('incBy', {val: value})
    }
  },

  getters: {
    getCount (state) { return state.count }
  }
})
