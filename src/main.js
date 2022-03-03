import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import "./index.css";
import VueSSE from "vue-sse";
import axios from "axios";

axios.defaults.baseURL = "/api/";

createApp(App)
  .use(VueSSE, {
    format: "json",
    polyfill: true,
    url: "/my-events-server",
    withCredentials: true,
  })
  .use(router)
  .mount("#app");
