package gametheory.assignment2.stratmirror;

import gametheory.assignment2.Player;

public class Strat implements Player {
  public void reset() {}
  public int move(int opponentLastMove, int xA, int xB, int xC) {
    return (opponentLastMove) % 3 + 1;
  }
  public String getEmail() { return ""; }
}
