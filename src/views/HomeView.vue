<template>
  <div class="flex grow bg-zinc-100">
    <div class="flex flex-col justify-start">
      <div class="flex flex-row w-screen justify-center">
        <div
          class="sm:text-xl md:text-2xl lg:text-3xl antialiased font-sans m-5"
        >
          The Scoreboard
        </div>
      </div>
      <ScoreTable :items="scoreboard" />
    </div>
  </div>
</template>

<script>
// @ is an alias to /src
import ScoreTable from "@/components/ScoreTable.vue";
import { Scoreboard } from "@/store.js";

export default {
  name: "HomeView",
  components: {
    ScoreTable,
  },
  data() {
    return {
      Scoreboard,
    };
  },
  async mounted() {},
  computed: {
    scoreboard() {
      //debugger;
      return this.Scoreboard.data.positions;
    },
  },
};
</script>
