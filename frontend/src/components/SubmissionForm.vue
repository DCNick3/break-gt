<template>
  <div class="flex flex-col">
    <div class="flex flex-col md:flex-row justify-center">
      <div class="flex flex-row justify-center md:justify-start">
        <div
          class="whitespace-nowrap sm:text-base md:text-xl lg:text-2xl md:pl-40 pr-2"
        >
          Upload your file here:
        </div>
      </div>
      <div class="flex flex-row justify-center md:justify-start">
        <input
          ref="file"
          v-on:change="fileUpload()"
          type="file"
          class="ml-20 md:m-0 pl-2"
        />
      </div>
    </div>
    <div v-if="file_uploading">
      <div
        class="flex flex-row justify-center w-screen text-gray-500 sm:text-base md:text-xl lg:text-2xl"
      >
        <div class="md:pl-20">Uploading...</div>
      </div>
    </div>
    <div v-else></div>
    <div v-if="file_uploaded">
      <div
        class="flex flex-row justify-center w-screen text-green-500 pl-5 md:pl-10 sm:text-base md:text-xl lg:text-2xl"
        v-if="success"
      >
        <div>It worked! Wait a bit for the results.</div>
      </div>
      <div
        class="flex flex-col justify-center text-rose-500 sm:text-base md:text-xl lg:text-2xl"
        v-else
      >
        <div class="flex flex-row justify-center w-screen">
          <div class="md:pl-20">File upload has failed. Try again.</div>
        </div>
        <div class="flex flex-row justify-start w-screen">
          <pre
            class="sm:text-sm md:text-base lg:text-sxl bg-zinc-100 ml-2 md:ml-10"
            >{{ error }}</pre
          >
        </div>
      </div>
    </div>
    <div v-else></div>
  </div>
</template>

<script>
import { ref } from "vue";
import CodeSubmissionAPI from "../api/code_submission.js";

export default {
  name: "SubmissionForm",
  props: {},
  data() {
    return {
      success: false,
      file_uploaded: false,
      error: undefined,
      file_uploading: false,
    };
  },
  setup() {
    const file = ref(null);
    return {
      file,
    };
  },
  methods: {
    fileUpload: async function () {
      const file = this.file;
      console.log("selected file", file.files);
      if (file.files.length == 0) {
        console.error("No file selected");
        return;
      }
      this.success = false;
      this.file_uploaded = false;
      this.file_uploading = true;
      let code = file.files[0];
      let fileContent = await code.text();
      let res = await CodeSubmissionAPI.create(fileContent);
      this.file_uploading = false;
      this.success = res.status;
      this.error = res.error;
      this.file_uploaded = true;
      file.value = "";
    },
  },
};
</script>
