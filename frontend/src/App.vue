<template>
  <div class="flex flex-col h-screen overflow-scroll">
    <SSEHandler></SSEHandler>
    <div class="flex flex-row w-screen justify-start bg-zinc-100">
      <div class="flex flex-row w-1/3 justify-start">
        <div
          class="text-sm md:text-xl pl-2"
          :class="{
            'text-green-700': time_is_great,
            'text-gray-900': time_is_ok,
            'text-yellow-700': time_is_bad,
            'text-rose-800': time_is_really_bad,
          }"
        >
          Tables updated {{ time_stringy }} ago
        </div>
      </div>
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
import { store, last_event_recieved_time } from "./store.js";
import AuthorizationAPI from "./api/check_authorization.js";
import axios from "axios";
import SSEHandler from "./components/SSEHandler.vue";
import TimeAgo from "javascript-time-ago";
import en from "javascript-time-ago/locale/en.json";
TimeAgo.addDefaultLocale(en);
const timeAgo = new TimeAgo("en-US");

export default {
  name: "App",
  data() {
    return {
      store,
      last_event_recieved_time,
      timeag: Date.now() - 1000,
      time_stringy: "1 day",
      interval_id: null,
      time_is_great: false,
      time_is_ok: false,
      time_is_bad: false,
      time_is_really_bad: false,
    };
  },
  computed: {
    is_anon() {
      return this.store.username === "Anonymous";
    },
  },
  async mounted() {
    this.updateOnce();
    this.updateTime();
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
    updateTime() {
      this.interval_id = setInterval(() => {
        this.timeag = this.last_event_recieved_time.data;
        this.time_is_great = Date.now() - this.timeag <= 5000;
        this.time_is_ok =
          Date.now() - this.timeag > 5000 && Date.now() - this.timeag <= 60000;
        this.time_is_bad =
          Date.now() - this.timeag > 60000 &&
          Date.now() - this.timeag <= 120000;
        this.time_is_really_bad = Date.now() - this.timeag > 120000;
        this.time_stringy = timeAgo.format(this.timeag, "mini");
      }, 5000);
    },
    updateOnce() {
      this.timeag = this.last_event_recieved_time.data;
      this.time_is_great = Date.now() - this.timeag <= 5000;
      this.time_is_ok =
        Date.now() - this.timeag > 5000 && Date.now() - this.timeag <= 60000;
      this.time_is_bad =
        Date.now() - this.timeag > 60000 && Date.now() - this.timeag <= 120000;
      this.time_is_really_bad = Date.now() - this.timeag > 120000;
      this.time_stringy = timeAgo.format(this.timeag, "mini");
    },
  },
  unmounted() {
    clearInterval(this.interval_id);
  },
  components: { SSEHandler },
};
</script>

<style>
.router-link-exact-active {
  @apply text-slate-900;
}
</style>
