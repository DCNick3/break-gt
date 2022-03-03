<template>
  <div class="flex grow submission bg-zinc-100">
    <div class="flex flex-col justify-start grow">
      <div class="flex flex-row w-screen justify-center">
        <div
          class="sm:text-xl md:text-2xl lg:text-3xl antialiased font-sans m-5"
        >
          Last round results
        </div>
      </div>
      <SubmissionTable :items="matches" class="h-3/6"> </SubmissionTable>
      <div class="flex flex-row justify-center">
        <SubmissionForm> </SubmissionForm>
      </div>
    </div>
  </div>
</template>

<script>
// @ is an alias to /src
import SubmissionForm from "@/components/SubmissionForm.vue";
import SubmissionTable from "@/components/SubmissionTable.vue";
import axios from "axios";
import { store, Matches } from "@/store.js";

export default {
  name: "AboutView",
  components: {
    SubmissionForm,
    SubmissionTable,
  },
  data() {
    return {
      store,
      auth_url: axios.defaults.baseURL + "login",
      Matches,
    };
  },
  mounted() {
    if (this.store.username === "Anonymous") {
      window.location.href = this.auth_url;
    }
  },
  computed: {
    matches() {
      return this.Matches.data.matches;
    },
  },
};
</script>
