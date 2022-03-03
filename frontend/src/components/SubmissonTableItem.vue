<template>
  <div
    class="flex flex-row sm:text-base md:text-xl lg:text-2xl antialiased font-sans w-full mt-[-2px]"
  >
    <div
      class="flex-initial px-1.5 py-1 text-left border-solid border-2 border-r-0 text-gray-900 border-yellow-500 bg-yellow-200 w-2/6"
    >
      {{ item.opponent_name }}
    </div>
    <div
      class="flex-initial px-1.5 py-1 text-center border-2 border-r-0 text-gray-700 border-yellow-500 bg-yellow-200 w-2/6"
    >
      {{ total_score }}
    </div>
    <div
      v-if="is_ok_my"
      class="flex-initial px-1.5 py-1 text-center border-2 border-r-0 text-green-800 border-yellow-500 bg-yellow-200 w-1/6"
    >
      {{ my_score }}
    </div>
    <div
      v-else
      class="flex-initial px-1.5 py-1 text-center border-2 border-r-0 text-green-800 border-yellow-500 bg-rose-400 w-1/6"
    >
      {{ my_score }}
    </div>
    <div
      v-if="is_ok_opp"
      class="flex-initial px-1.5 py-1 text-center border-2 text-rose-800 border-yellow-500 bg-yellow-200 w-1/6"
    >
      {{ opp_score }}
    </div>
    <div
      v-else
      class="flex-initial px-1.5 py-1 text-center border-2 text-rose-800 border-yellow-500 bg-rose-400 w-1/6"
    >
      {{ opp_score }}
    </div>
  </div>
</template>

<script>
export default {
  name: "SubmissionTableItem",
  props: {
    item: Object,
  },
  data() {
    return { isOkMy: Boolean, isOkOpp: Boolean };
  },
  computed: {
    total_score() {
      return parseFloat(this.item.opponent_scoreboard_score).toFixed(3);
    },
    my_score() {
      if ("Ok" in this.item.your_result)
        return parseFloat(this.item.your_result.Ok).toFixed(3);
      return this.item.your_result.Err;
    },
    opp_score() {
      if ("Ok" in this.item.opponent_result)
        return parseFloat(this.item.opponent_result.Ok).toFixed(3);
      return this.item.your_result.Err;
    },
    is_ok_my() {
      if ("Ok" in this.item.your_result) return true;
      return false;
    },
    is_ok_opp() {
      if ("Ok" in this.item.opponent_result) return true;
      return false;
    },
  },
};
</script>
