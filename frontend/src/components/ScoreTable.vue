<template>
  <div class="flex-auto flex flex-row justify-center w-screen">
    <div
      class="flex flex-col flex-nowrap justify-start overflow-y-auto w-4/5 p-2"
    >
      <TableItem v-for="item in items" :key="item.id" :item="item"></TableItem>
    </div>
  </div>
</template>

<script>
import TableItem from "./TableItem.vue";
export default {
  name: "ScoreTable",
  props: {
    items: Array,
  },
  components: { TableItem },
};
</script>
