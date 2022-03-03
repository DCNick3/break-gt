import axios from "axios";

export default {
  name: "AuthorizationAPI",
  async show(params) {
    return await axios.get("/me", {
      withCredentials: true,
      params: params,
    });
  },
};
