import { reactive } from "vue";

export const store = reactive({
  username: "",
});

export const Scoreboard = reactive({
  data: {
    datetime: "2022-03-03T16:49:41.114686182Z",
    positions: [
      ["This is placeholder", "10091"],
      ["If it does not disappear check internet connection", "10091"],
      ["Or maybe allow scripts", "10091"],
      ["Or wait a bit till page loads", "10091"],
      ["If it does not load, write to @DCNick3", "10091"],
      ["Maybe his backend died or smth", "10091"],
    ],
  },
});

export const Matches = reactive({
  data: {
    round_time: "2022-03-03T16:47:21.075808115Z",
    matches: [
      {
        your_result: { Ok: 374.85777513421453 },
        opponent_result: { Ok: 2.3105857863000487 },
        opponent_name: "Placeholder",
        opponent_scoreboard_score: 1.1552928931500244,
      },
      {
        your_result: { Ok: 398.1198786663252 },
        opponent_result: { Ok: 2.3105857863000487 },
        opponent_name: "If this stays this way write to @unb0und",
        opponent_scoreboard_score: 1.1552928931500244,
      },
      {
        your_result: { Ok: 398.1198786663252 },
        opponent_result: { Ok: 2.3105857863000487 },
        opponent_name: "Maybe the backend is down",
        opponent_scoreboard_score: 1.1552928931500244,
      },
      {
        your_result: { Ok: 398.1198786663252 },
        opponent_result: { Ok: 2.3105857863000487 },
        opponent_name: "Or something is wrong with your internet/scripts",
        opponent_scoreboard_score: 1.1552928931500244,
      },
    ],
  },
});
