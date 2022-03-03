<template>
  <div class="flex flex-row justify-center w-screen">
    <div class="flex flex-col flex-nowrap justify-start overflow-y-auto w-4/5">
      <div
        class="flex flex-row font-bold sm:text-sm md:text-base lg:text-xl antialiased font-sans w-full"
      >
        <div
          class="flex-initial px-1.5 py-1 text-left border-solid bg-yellow-200 w-2/6"
        >
          Opponent
        </div>
        <div class="flex-initial px-1.5 py-1 text-center bg-yellow-200 w-2/6">
          Opponents full score
        </div>
        <div class="flex-initial px-1.5 py-1 text-center bg-yellow-200 w-1/6">
          Your match score
        </div>
        <div class="flex-initial px-1.5 py-1 text-center bg-yellow-200 w-1/6">
          Opp match score
        </div>
      </div>
      <SubmissionTableItem
        v-for="item in items"
        :key="item.id"
        :item="item"
      ></SubmissionTableItem>
    </div>
  </div>
</template>

<script>
import SubmissionTableItem from "./SubmissonTableItem.vue";
export default {
  name: "SubmissionTable",
  props: {
    items: Array,
  },
  components: { SubmissionTableItem },
};
</script>
