<template>
  <div class="flex flex-col h-screen overflow-scroll">
    <SSEHandler></SSEHandler>
    <div class="flex flex-row w-screen justify-start bg-zinc-100">
      <div class="flex flex-row w-1/3"></div>
      <div class="flex flex-row w-1/3 justify-center">
        <nav class="text-center text-sm md:text-2xl text-green-800">
          <router-link to="/">Scoreboard</router-link> |
          <router-link to="/submission">Submit</router-link>
        </nav>
      </div>
      <div class="flex flex-row w-1/3 justify-end">
        <div class="flex flex-col justify-start">
          <div class="text-sm md:text-xl pr-2 md:pr-4">
            Welcome, {{ store.username }}.
          </div>
          <div v-if="is_anon" class="flex flex-row justify-end mr-4 md:mr-6">
            <button
              @click="onLoginClick"
              class="text-sm md:text-xl font-bold bg-green-600 hover:bg-green-800 text-zinc-100 py-1 px-2 rounded"
            >
              Login
            </button>
          </div>
        </div>
      </div>
    </div>
    <router-view />
  </div>
</template>

<script>
import { store } from "./store.js";
import AuthorizationAPI from "./api/check_authorization.js";
import axios from "axios";
import SSEHandler from "./components/SSEHandler.vue";

export default {
  name: "App",
  data() {
    return { store };
  },
  computed: {
    is_anon() {
      return this.store.username === "Anonymous";
    },
  },
  async mounted() {
    let res = await AuthorizationAPI.show();
    if (res.data.user === null) {
      this.store.username = "Anonymous";
    } else {
      this.store.username = res.data.user.username;
    }
  },
  methods: {
    onLoginClick() {
      window.location.href = axios.defaults.baseURL + "login";
    },
  },
  components: { SSEHandler },
};
</script>

<style>
.router-link-exact-active {
  @apply text-emerald-500;
}
</style>
