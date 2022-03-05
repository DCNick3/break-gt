import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import "./index.css";
import VueSSE from "vue-sse";
import axios from "axios";

const API_URL =
  process.env.NODE_ENV === "production"
    ? "/api/"
    : "http://localhost:8081/api/";
console.log("Api URL is " + API_URL);

axios.defaults.baseURL = API_URL;

createApp(App)
  .use(VueSSE, {
    format: "json",
    polyfill: true,
    forcePolyfill: true,
    url: "/my-events-server",
    withCredentials: true,
  })
  .use(router)
  .mount("#app");
