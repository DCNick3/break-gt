<template>
  <div></div>
</template>

<script>
import { Scoreboard, Matches, last_event_recieved_time } from "@/store.js";
import axios from "axios";
let sseClient;

export default {
  name: "SSEHandler",
  data() {
    return { Scoreboard, Matches, last_event_recieved_time };
  },
  async mounted() {
    sseClient = this.$sse.create({
      url: axios.defaults.baseURL + "events",
      format: "json",
      withCredentials: true,
      polyfill: true,
    });

    // Catch any errors (ie. lost connections, etc.)
    sseClient.on("error", (e) => {
      console.error("lost connection or failed to parse!", e);
    });

    // Handle messages without a specific event
    sseClient.on("message", this.handleMessage);

    sseClient.on("scoreboard", this.handleScoreboard);

    sseClient.on("matches", this.handleMatches);

    sseClient
      .connect()
      // eslint-disable-next-line
      .then((sse) => {
        console.log("We're connected!");
      })
      .catch((err) => {
        console.error("Failed to connect to server", err);
      });
  },
  methods: {
    handleScoreboard(ScoreboardMessage) {
      this.last_event_recieved_time.data = Date.now();
      this.Scoreboard.data = ScoreboardMessage;
    },
    handleMatches(MatchesMessage) {
      this.last_event_recieved_time.data = Date.now();
      this.Matches.data = MatchesMessage;
    },
    handleMessage(message, lastEventId) {
      console.warn("Received a message w/o an event!", message, lastEventId);
    },
  },
  beforeUnmount() {
    sseClient.disconnect();
  },
};
</script>
