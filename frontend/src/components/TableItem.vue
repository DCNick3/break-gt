<template>
  <div
    class="flex flex-row sm:text-base md:text-xl lg:text-2xl antialiased font-sans w-full mt-[-2px]"
  >
    <div
      class="flex-initial px-1.5 py-1 text-left border-solid border-2 border-r-0 rounded-l-lg border-blue-500 bg-sky-200 w-2/3"
    >
      {{ item[0] }}
    </div>
    <div
      class="flex-initial px-1.5 py-1 text-center border-solid border-2 rounded-r-lg border-blue-500 bg-sky-200 w-1/3"
    >
      {{ score }}
    </div>
  </div>
</template>

<script>
export default {
  name: "TableItem",
  props: {
    item: Object,
  },
  computed: {
    score() {
      return parseFloat(this.item[1]).toFixed(3);
    },
  },
};
</script>
