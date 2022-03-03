import axios from "axios";

export default {
  name: "CodeSubmissionAPI",
  async index(params) {
    return axios.get("/get", {
      withCredentials: true,
      params: params,
    });
  },

  async create(data) {
    // eslint-disable-next-line
    return await axios
      .post("/submit", data, {
        withCredentials: true,
      })
      // eslint-disable-next-line
      .then((r) => {
        return { status: true, error: undefined };
      })
      .catch(function (error) {
        if (error.response) {
          console.log(error.response.data);
          console.log(error.response.status);
          console.log(error.response.headers);
          return { status: false, error: error.response.data[1] };
        }
        return { status: false, error: error.message };
      });
  },
};
