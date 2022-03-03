import axios from "axios";

export default {
  name: "ScoreBoardAPI",
  async index(params) {
    return axios.get("/scoreboard", {
      params: params,
    });
  },
};
